-- Insert sample data
insert into todos (id, title, description) values ('01JE81ECXT8WE0FTRD94ST3TVV', '오늘 할일', '운동, 독서를 하자.');
insert into todos (id, title, description) values ('01GDT91MB0SGG49T974GX2A5G9', '내일 할일', '은행에 가서 공납금을 납부하자.');
update todos set status_id = '01JDYBNWTNPEX65JFNY3HTH15H', updated_at = current_timestamp where id = '01JE81ECXT8WE0FTRD94ST3TVV';
select table_name, constraint_name, constraint_type
from   information_schema.table_constraints
where  table_name = 'todos';