create table if not exists alert_types (
    name text not null,
    alert_level smallint not null,
    created_at timestamp without time zone default now(),
    updated_at timestamp without time zone default now(),
	constraint unique_alert_name unique (name)
);

insert into alert_types (name, alert_level) values
('Kidnap', 1),
('Theft', 1),
('Assault', 1),
('Other Emergency', 1),
('Suspicious Person Spotted', 2),
('Garage Sale', 3);

create table if not exists alerts (
    id bigserial primary key,
    alert_type text not null references alert_types (name),
    description text not null default 'none provided',
    place text not null,
    latitude real not null,
    longitude real not null,
    display_email bool not null default true,
    display_phone bool not null default true,
    track_location bool not null default false,
    created_by text not null default 'internal',
    is_resolved bool not null default false,
    created_at timestamp without time zone default now(),
    updated_at timestamp without time zone default now()
);

create index alert_location_index on alerts (is_resolved, latitude, longitude);