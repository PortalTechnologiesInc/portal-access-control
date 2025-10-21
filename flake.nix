{
  description = "Rust toolchain flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    let
      nixosModule =
        {
          config,
          lib,
          pkgs,
          ...
        }:

        with lib;

        let
          cfg = config.services.access-control-service;
        in
        {
          options.services.access-control-service = {
            enable = mkEnableOption "Access control service for access with portal";

            package = mkOption {
              type = types.package;
              description = "This is the package/derivation to use.";
            };

            port = mkOption {
              type = types.port;
              default = 7007;
              description = "Port to listen on";
            };

            databaseUrl = mkOption {
              type = types.str;
              default = "postgres://localhost:5433/access_control_db";
              description = "Url the service will use to connect to postgres db";
            };

            host = mkOption {
              type = types.str;
              default = "127.0.0.1";
              description = "Host to bind the server to";
            };

            profile = mkOption {
              type = types.str;
              default = "prod";
              description = "rocket environment configuration";
            };

            user = mkOption {
              type = types.str;
              default = "access-control-service";
              description = "User to run the service as";
            };

            group = mkOption {
              type = types.str;
              default = "access-control-service";
              description = "Group to run the service as";
            };
          };

          config =
            let
              # Combine all environment variables
              envConfig = {
                ROCKET_ADDRESS = cfg.host;
                ROCKET_PORT = toString cfg.port;
                DATABASE_URL = cfg.databaseUrl;
                ROCKET_PROFILE = cfg.profile;
              };
            in
            mkIf cfg.enable {
              systemd.services.access-control-service = {
                description = "Access control service";
                wantedBy = [ "multi-user.target" ];
                after = [ "network.target" ];

                environment = envConfig;

                serviceConfig = {
                  ExecStartPre = "${pkgs.sqlx-cli}/bin/sqlx migrate run --source ${cfg.package}/migrations";
                  ExecStart = "${cfg.package}/bin/backend";
                  Restart = "always";
                  ProtectSystem = "strict";
                  ProtectHome = true;
                  PrivateTmp = true;
                  NoNewPrivileges = true;
                  User = cfg.user;
                  Group = cfg.group;
                };
              };

              users.users.${cfg.user} = {
                isSystemUser = true;
                group = cfg.group;
              };
              users.groups.${cfg.group} = {};
            };
        };
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rust = (
          pkgs.rust-bin.nightly.latest.default.override {
            extensions = [
              "rust-src" # for rust-analyzer
            ];
          }
        );

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust;
          rustc = rust;
        };

        backendPackage = rustPlatform.buildRustPackage {
          pname = "access-control-backend";
          version = "0.1.0";
          src = ./backend;
          cargoHash = "sha256-XbSDHKeR210Kzkuac5XKoqIUkK5HfijDyiifN/S21Yc=";
          cargoLock.lockFile = ./backend/Cargo.lock;

          buildInputs = with pkgs; [
            postgresql
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
            openssl
            sqlx-cli
          ];

          DATABASE_URL = "postgres://localhost:5433/access_control_db";

          # this is necessary to avoid buildRustPackage to do cargo check after build
          doCheck = false;

          preBuild = ''
            export PGDATA="$PWD/postgres-data"
            export PGHOST="$PWD/postgres"
            export PGPORT="5433"
            export PGDATABASE="access_control_db"

            if [ ! -d "$PGDATA" ]; then
              ${pkgs.postgresql}/bin/initdb -D "$PGDATA"
              echo "unix_socket_directories = '$PGHOST'" >> "$PGDATA/postgresql.conf"
              echo "port = $PGPORT" >> "$PGDATA/postgresql.conf"
            fi

            # Create socket directory
            mkdir -p "$PGHOST"

            # Start PostgreSQL
            ${pkgs.postgresql}/bin/postgres -D "$PGDATA" &
            POSTGRES_PID=$!

            echo $POSTGRES_PID
            # Wait for PostgreSQL to start
            sleep 2

            # Create database if it doesn't exist
            ${pkgs.postgresql}/bin/createdb "$PGDATABASE" 2>/dev/null || true
            cargo sqlx database create
            cargo sqlx migrate run
          '';

          postBuild = ''
            export PGDATA="$PWD/postgres-data"
            ${pkgs.postgresql}/bin/pg_ctl stop -D "$PGDATA"
          '';

	  postInstall = ''
            mkdir -p $out/migrations
            cp migrations/* $out/migrations
	  '';
        };
      in
      {
        devShells = {
          default = pkgs.mkShell {
            name = "access-control-backend";
            buildInputs = with pkgs; [
              rust
              rust-analyzer
              openssl
              pkg-config
              protobuf
              docker
              sqlx-cli
            ];
          };
        };

        packages = {
          default = backendPackage;
        };

        checks = {
          vm-test = pkgs.testers.runNixOSTest {
            name = "access-control-service-vm-test";

            nodes.machine = { config, pkgs, ... }: {
              imports = [ nixosModule ];

              services.postgresql = {
                enable = true;
                ensureDatabases = [ "access_control_db" ];
                ensureUsers = [
                  {
                    name = "access_control_db";
                    ensureDBOwnership = true;
                  }
                ];
                authentication = pkgs.lib.mkOverride 10 ''
                  local all all trust
                  host all all 127.0.0.1/32 trust
                  host all all ::1/128 trust
                '';
              };

              services.portal-profile-service = {
                enable = true;
                package = backendPackage;
                host = "0.0.0.0";
                port = 7007;
                databaseUrl = "postgres://profile_db@localhost/profile_db";
              };

              # Open firewall for the service
              networking.firewall.allowedTCPPorts = [ 7007 ];
            };

            testScript = ''
              machine.start()
              machine.wait_for_unit("postgresql.service")
              machine.wait_for_unit("access-control-service.service")

              # Wait a bit more for the service to fully start
              machine.sleep(5)

              # Test the health check endpoint
              machine.succeed("curl -f http://localhost:7007/health_check")

              print("✅ Access control service is running and health check passed!")
            '';
          };
        };
      }
    )
    // {
      nixosModules.myModule = nixosModule;
    };
}
