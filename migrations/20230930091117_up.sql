-- Setup tables
create table if not exists users (
    id varchar(26) not null,
    username varchar(255) not null,
    email varchar(255) not null,
    password varchar(255) not null,
    constraint pk_user_id primary key (id)
);

create table if not exists todo_statuses (
    id varchar(26) not null,
    code varchar(255) not null,
    name varchar(255) not null,
    constraint pk_todo_statuses_id primary key (id)
);

create table if not exists todos (
    id varchar(26) not null,
    title varchar(255) not null,
    description text not null,
    status_id varchar(26) not null default '01JDW75BSGY2T185G842JNTWS7',
    created_at timestamp with time zone not null default current_timestamp,
    updated_at timestamp with time zone not null default current_timestamp,
    constraint pk_todos_id primary key (id),
    constraint fk_todos_status_id_todo_statuses_id foreign key (status_id) references todo_statuses (id)
);
