-- Your SQL goes here
create table tags
(
    id      serial primary key,
    user_id int     not null references users (id),
    name    varchar not null
);
