create table if not exists users (
    id bigserial primary key,
    name text not null,
    email text not null,
    password text not null,
    phone text,
    verified bool not null default false,
    created_at timestamp without time zone default now(),
    updated_at timestamp without time zone default now(),
    constraint unique_email unique (email)
);

create index user_email_index on users (email);