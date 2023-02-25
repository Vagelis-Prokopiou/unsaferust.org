CREATE TABLE IF NOT EXISTS error_log
(
    id         serial primary key,
    error      text not null,
    created_at DATE not null default CURRENT_DATE
);