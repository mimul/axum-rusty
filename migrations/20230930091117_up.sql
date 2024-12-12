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

insert into todo_statuses (id, code, name) values ('01JDW75BSGY2T185G842JNTWS7', 'new', '신규') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JDYBNWTNPEX65JFNY3HTH15H', 'working', '착수') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JDYE4RD1EEQTNP90EKM99QG5', 'waiting', '미착수') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JESRK1CC4TEVB53JWDCDA8JX', 'done', '완료') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JESRKPH2G026RKV29GNTY5R0', 'discontinued', '중단') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JESRM34FRG9K3D8M7KXRCTDX', 'pending', '보류') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JESRMCSRT1N81P95JG46N0K0', 'deleted', '삭제') on conflict do nothing;

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
