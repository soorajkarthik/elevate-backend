create table if not exists locations (
    user_id bigint not null references users (id) on delete cascade,
    latitude real not null,
    longitude real not null,
    created_at timestamp without time zone default now(),
    updated_at timestamp without time zone default now(),
    constraint single_location_per_user unique (user_id)
);

create index lat_long_index on locations (latitude, longitude);

create or replace function calculate_distance(lat1 real, lon1 real, lat2 real, lon2 real)
returns real AS $dist$
    declare
        dist real = 0;
        radlat1 real;
        radlat2 real;
        theta real;
        radtheta real;
    begin
        if lat1 = lat2 and lon1 = lon2
            then return dist;
        else
            radlat1 = pi() * lat1 / 180;
            radlat2 = pi() * lat2 / 180;
            theta = lon1 - lon2;
            radtheta = pi() * theta / 180;
            dist = sin(radlat1) * sin(radlat2) + cos(radlat1) * cos(radlat2) * cos(radtheta);

            if dist > 1 then dist = 1; end if;

            dist = acos(dist);
            dist = dist * 180 / pi();
            dist = dist * 60 * 1.1515;

            return dist;
        end if;
    end;
$dist$ language plpgsql;