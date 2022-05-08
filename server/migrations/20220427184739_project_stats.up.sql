CREATE TABLE IF NOT EXISTS project_stats
(
    project_id   int  not null,
    code_lines   int  not null,
    unsafe_lines int  not null,
    created_at   DATE not null default CURRENT_DATE,
    updated_at   DATE not null default CURRENT_DATE,
    UNIQUE (project_id, unsafe_lines),
    CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES projects (id)
);