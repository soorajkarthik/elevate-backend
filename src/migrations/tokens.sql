create table if not exists tokens (
    token text primary key,
    token_type text not null default 'internal use',
    created_for text not null default 'internal',
    created_at timestamp without time zone default now(),
    updated_at timestamp without time zone default now()
);

create table if not exists firebase_device_tokens (
    user_id bigint not null references users (id) on delete cascade,
    token text not null,
    created_at timestamp without time zone default now(),
    updated_at timestamp without time zone default now()
);

create index token_user_id_index on firebase_device_tokens (user_id);