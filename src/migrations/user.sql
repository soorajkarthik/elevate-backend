create table if not exists users (
    id bigserial primary key,
    name text not null,
    email text not null,
    password text not null,
    phone text,
    latitude real,
    longitude real,
    accepted_location_tracking not null default false,
    created_at timestamp without time zone default now(),
    updated_at timestamp without time zone default now(),
    constraint unique_email unique (email)
)

create index location_index on users (latitude, longitude)