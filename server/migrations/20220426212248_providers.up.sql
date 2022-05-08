CREATE TABLE IF NOT EXISTS providers
(
    id   serial primary key,
    url  varchar(255) not null unique
);