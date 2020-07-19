create table if not exists users (
    id bigserial primary key,
    name text not null,
    email text not null,
    password text not null,
    phone text,
    verified bool not null default false,
    accepted_location_tracking bool not null default false,
    created_at timestamp without time zone default now(),
    updated_at timestamp without time zone default now(),
    constraint unique_email unique (email)
)

create table if not exists tokens (
    token text not null,
    token_type text not null default 'internal use',
    created_for text not null default 'internal',
    created_at timestamp without time zone default now(),
    updated_at timestamp without time zone default now()
)