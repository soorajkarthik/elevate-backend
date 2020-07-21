create table if not exists tokens (
    token text primary key,
    token_type text not null default 'internal use',
    created_for text not null default 'internal',
    created_at timestamp without time zone default now(),
    updated_at timestamp without time zone default now()
);