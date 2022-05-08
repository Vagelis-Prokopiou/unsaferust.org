CREATE TABLE IF NOT EXISTS projects
(
    id          serial       not null primary key,
    provider_id int          not null,
    namespace   varchar(255) not null,
    name        varchar(255) not null unique,
    CONSTRAINT fk_provider FOREIGN KEY (provider_id) REFERENCES providers (id)
);