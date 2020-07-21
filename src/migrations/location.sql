create table if not exists locations (
    user_id bigint not null references users (id) on delete cascade,
    latitude real not null,
    longitude real not null,
    created_at timestamp without time zone default now(),
    updated_at timestamp without time zone default now(),
    constraint single_location_per_user unique (user_id)
);

create index lat_long_index on locations (latitude, longitude);