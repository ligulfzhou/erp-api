-- Add up migration script here
-- 商品表
create table goods
(
    id       SERIAL,
    goods_no text, -- 类目编号
    image    text, -- 图片
    name     text, -- 名称
    plating  text, -- 电镀
    notes    text  -- 备注
);

insert into goods (goods_no, image, name, plating, notes)
values ('goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_1', 'plating_1', 'notes_1');
insert into goods (goods_no, image, name, plating, notes)
values ('goods_no_2', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_2', 'plating_2', 'notes_2');
insert into goods (goods_no, image, name, plating, notes)
values ('goods_no_3', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_3', 'plating_3', 'notes_3');
insert into goods (goods_no, image, name, plating, notes)
values ('goods_no_4', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_4', 'plating_4', 'notes_4');
insert into goods (goods_no, image, name, plating, notes)
values ('goods_no_5', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_5', 'plating_5', 'notes_5');

-- sku表
create table skus
(
    id       SERIAL,
    goods_id integer, -- 类目ID
    image    text,    -- 商品图片
    name     text,    -- 商品名称
    goods_no text not null,    -- 产品编号 (暂时没有)
    sku_no   text not null,    -- sku编号
    plating  text,    -- 电镀
    color    text,    -- 颜色
    notes    text     -- 备注
);

insert into skus (goods_id, name, image, goods_no, sku_no, color, notes)
values
(1, 'name_1',  'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_1_sku_1', 'blue', 'notes....'),
(1, 'name_2', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_1_sku_2', 'blue', 'notes....'),
(1, 'name_3', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_1_sku_3', 'blue', 'notes....'),
(1, 'name_4', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_1_sku_4', 'blue', 'notes....'),
(1, 'name_5', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_1_sku_5', 'blue', 'notes....'),
(1, 'name_6', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_1_sku_6', 'blue', 'notes....'),
(2, 'name_7', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_2_sku_1', 'blue', 'notes....'),
(2, 'name_8', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_2_sku_2', 'blue', 'notes....'),
(2, 'name_9', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_2_sku_3', 'blue', 'notes....'),
(2, 'name_10', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_2_sku_4', 'blue', 'notes....'),
(2, 'name_11', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_2_sku_5', 'blue', 'notes....'),
(2, 'name_12', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_no_1', 'goods_2_sku_6', 'blue', 'notes....');

-- 客户
create table customers
(
    id          serial,
    customer_no text, -- 客户编号
    name        text, -- 名称
    address     text, -- 地址
    phone       text, -- 电话
    notes       text  -- 备注
);
insert into customers (customer_no, name, address, phone, notes) values ('customer_no_1', 'customer_name_1', 'address.....address', '110', '这是客户1');
insert into customers (customer_no, name, address, phone, notes) values ('customer_no_2', 'customer_name_2', 'address.....address', '220', '这是客户2');
insert into customers (customer_no, name, address, phone, notes) values ('customer_no_3', 'customer_name_3', 'address.....address', '330', '这是客户3');
insert into customers (customer_no, name, address, phone, notes) values ('customer_no_4', 'customer_name_4', 'address.....address', '440', '这是客户4');
insert into customers (customer_no, name, address, phone, notes) values ('customer_no_5', 'customer_name_5', 'address.....address', '550', '这是客户5');

-- 订单表
create table orders
(
    id            serial,
    customer_id   integer not null, -- 客户ID
    order_no      text not null,    -- 订单编号
    order_date    integer not null, -- 订货日期
    delivery_date integer  -- 交货日期
);
insert into orders (customer_id, order_no, order_date, delivery_date)
VALUES (1, 'order_no_1', 1691558739, 1691568739),
       (2, 'order_no_2', 1691558739, 1691568739),
       (3, 'order_no_3', 1691558739, 1691568739);

-- 订单商品表
create table order_goods
(
    id  serial,
    order_id integer not null,
    goods_no text not null,
    package_card     text,
    package_card_des text,
);

-- 订单sku表
create table order_items
(
    id               serial,
    order_id         integer not null, -- 订单ID
    sku_id           integer not null, -- 商品ID
    order_goods_id   integer not null,
--    package_card     text,    -- 包装卡片    （存在大问题）
--    package_card_des text,    -- 包装卡片说明 （存在大问题）
    count            integer not null, -- 数量
    unit             text,    --单位
    unit_price       integer, -- 单价
    total_price      integer, -- 总价/金额
    notes            text     -- 备注
);

insert into order_items (order_id, sku_id, package_card, package_card_des, count, unit)
values (1, 1, 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'adsfasdfasdf', 10, '个'),
(1, 2, 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'adsfasdfasdf', 10, '个'),
(1, 3, 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'adsfasdfasdf', 10, '个');

-- 订单sku的的材料单 * N
create table order_item_materials
(
    id            serial,
    order_id      integer, -- 订单ID
    order_item_id integer, -- 订单商品ID
    name          text not null,    -- 材料名称
    color         text not null,    -- 材料颜色
--     material_id   integer, -- 材料ID  (暂时先不用)
    single        integer, -- 单数      ？小数
    count         integer, -- 数量      ？小数
    total         integer, -- 总数(米)  ? 小数
    stock         integer, -- 库存 ?
    debt          integer, -- 欠数
    notes         text     -- 备注
);

-- create table item
-- (
--     id         SERIAL,
--     image      text,
--     name       text,
--     plating    text,
--     num        integer,
--     size       text,
--     buy_price  integer,
--     sell_price integer,
--     memo       text
-- );

-- -- 材料表
-- create table material
-- (
--     id    serial,
--     name  text, -- 材料名
--     color text, -- 材料颜色
--     info  text  -- 材料说明
-- );

-- -- todo: 库存何时减少？ 在仓库确认之后？
-- create table material_stock
-- (
--     id          serial,
--     material_id integer,
--     buy_price   integer,
--     count       integer
-- );

-- 部门
create table departments
(
    id            serial,
    department_id integer,
    name          text,   -- 部门名称
    index         integer -- 流程位续
);

-- 账号
create table accounts
(
    id            serial,
    name          text,
    account       text,
    password      text,
    department_id integer
);

-- create index idx_index on department (`index`);
