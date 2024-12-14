ALTER TABLE users ADD CONSTRAINT unique_username UNIQUE (username);

insert into todo_statuses (id, code, name) values ('01JDW75BSGY2T185G842JNTWS7', 'new', '신규') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JDYBNWTNPEX65JFNY3HTH15H', 'working', '착수') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JDYE4RD1EEQTNP90EKM99QG5', 'waiting', '미착수') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JESRK1CC4TEVB53JWDCDA8JX', 'done', '완료') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JESRKPH2G026RKV29GNTY5R0', 'discontinued', '중단') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JESRM34FRG9K3D8M7KXRCTDX', 'pending', '보류') on conflict do nothing;
insert into todo_statuses (id, code, name) values ('01JESRMCSRT1N81P95JG46N0K0', 'deleted', '삭제') on conflict do nothing;

