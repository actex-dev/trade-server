use sea_orm_migration::prelude::*;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // admins
        conn.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"CREATE TABLE IF NOT EXISTS admins (
                id uuid PRIMARY KEY,
                email_address text NOT NULL,
                password text NOT NULL,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL,
                deleted_at timestamptz
            );"#,
        )).await?;

        // users
        conn.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"CREATE TABLE IF NOT EXISTS users (
                id uuid PRIMARY KEY,
                personal_first_name text NOT NULL,
                personal_second_name text NOT NULL,
                personal_email_address text NOT NULL UNIQUE,
                personal_user_roles text[] NOT NULL,
                personal_profile_image text,
                personal_username text,
                password text NOT NULL,
                peripheral_authentication_code text,
                peripheral_authentication_token text,
                peripheral_timeout timestamptz,
                peripheral_is_banned boolean NOT NULL,
                peripheral_is_verified boolean NOT NULL,
                verification_code text NOT NULL,
                verification_timeout bigint,
                setting_custom_setting_default_theme text,
                setting_custom_setting_is_accepting_request boolean NOT NULL,
                setting_subscription_price_id text,
                setting_subscription_product_id text,
                setting_subscription_status text NOT NULL,
                setting_subscription_start_date timestamptz,
                setting_subscription_end_date timestamptz,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL,
                deleted_at timestamptz
            );"#,
        )).await?;

        // organizations
        conn.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"CREATE TABLE IF NOT EXISTS organizations (
                id uuid PRIMARY KEY,
                name text NOT NULL,
                description text NOT NULL,
                template jsonb NOT NULL,
                project_template jsonb NOT NULL,
                stage text NOT NULL,
                status text NOT NULL,
                members integer NOT NULL,
                creator_id uuid NOT NULL REFERENCES users(id),
                settings jsonb NOT NULL,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL,
                deleted_at timestamptz
            );"#,
        )).await?;

        // organization_users
        conn.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"CREATE TABLE IF NOT EXISTS organization_users (
                id uuid PRIMARY KEY,
                user_id uuid NOT NULL REFERENCES users(id),
                organization_id uuid NOT NULL REFERENCES organizations(id),
                dashboards jsonb NOT NULL,
                role text NOT NULL,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL,
                deleted_at timestamptz
            );"#,
        )).await?;

        // projects
        conn.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"CREATE TABLE IF NOT EXISTS projects (
                id uuid PRIMARY KEY,
                organization_id uuid NOT NULL REFERENCES organizations(id),
                name text NOT NULL,
                description text,
                creator_id uuid NOT NULL REFERENCES users(id),
                settings jsonb NOT NULL,
                status text NOT NULL,
                priority text NOT NULL,
                start_date timestamptz,
                end_date timestamptz,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL,
                deleted_at timestamptz
            );"#,
        )).await?;

        // project_users
        conn.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"CREATE TABLE IF NOT EXISTS project_users (
                id uuid PRIMARY KEY,
                organization_id uuid NOT NULL REFERENCES organizations(id),
                user_id uuid NOT NULL REFERENCES users(id),
                organization_user_id uuid NOT NULL REFERENCES organization_users(id),
                project_id uuid NOT NULL REFERENCES projects(id),
                role text NOT NULL,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL,
                deleted_at timestamptz
            );"#,
        )).await?;

        // integrations
        conn.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"CREATE TABLE IF NOT EXISTS integrations (
                id uuid PRIMARY KEY,
                type text NOT NULL,
                category text NOT NULL,
                integration_status text NOT NULL,
                name text,
                credentials_access_key_id text,
                credentials_secret_access_key text,
                credentials_region text,
                oauth2_access_token text NOT NULL,
                oauth2_token_type text NOT NULL,
                oauth2_refresh_token text,
                oauth2_expiry timestamptz,
                oauth2_expires_in integer,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL,
                deleted_at timestamptz
            );"#,
        )).await?;

        // billings
        conn.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"CREATE TABLE IF NOT EXISTS billings (
                id uuid PRIMARY KEY,
                organization_id uuid NOT NULL REFERENCES organizations(id),
                amount numeric NOT NULL,
                started_at timestamptz NOT NULL,
                ended_at timestamptz NOT NULL,
                activities jsonb NOT NULL,
                is_paid boolean NOT NULL,
                created_at timestamptz NOT NULL,
                updated_at timestamptz NOT NULL,
                deleted_at timestamptz
            );"#,
        )).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Drop in order to satisfy FKs
        for stmt in [
            r#"DROP TABLE IF EXISTS project_users CASCADE;"#,
            r#"DROP TABLE IF EXISTS projects CASCADE;"#,
            r#"DROP TABLE IF EXISTS organization_users CASCADE;"#,
            r#"DROP TABLE IF EXISTS billings CASCADE;"#,
            r#"DROP TABLE IF EXISTS integrations CASCADE;"#,
            r#"DROP TABLE IF EXISTS organizations CASCADE;"#,
            r#"DROP TABLE IF EXISTS users CASCADE;"#,
            r#"DROP TABLE IF EXISTS admins CASCADE;"#,
        ] {
            conn.execute(Statement::from_string(DatabaseBackend::Postgres, stmt)).await?;
        }

        Ok(())
    }
}