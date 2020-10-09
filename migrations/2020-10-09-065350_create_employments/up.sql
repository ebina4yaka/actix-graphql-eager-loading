-- Your SQL goes here
create TABLE employments
(
    id         serial primary key,
    user_id    int not null references users (id),
    company_id int not null references companies (id)
);
