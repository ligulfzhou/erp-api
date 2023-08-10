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
values ('goods_no_1', 'http://baidu.com', 'goods_1', 'plating_1', 'notes_1');
insert into goods (goods_no, image, name, plating, notes)
values ('goods_no_2', 'http://baidu.com', 'goods_2', 'plating_2', 'notes_2');
insert into goods (goods_no, image, name, plating, notes)
values ('goods_no_3', 'http://baidu.com', 'goods_3', 'plating_3', 'notes_3');
insert into goods (goods_no, image, name, plating, notes)
values ('goods_no_4', 'http://baidu.com', 'goods_4', 'plating_4', 'notes_4');
insert into goods (goods_no, image, name, plating, notes)
values ('goods_no_5', 'http://baidu.com', 'goods_5', 'plating_5', 'notes_5');

-- sku表
create table skus
(
    id       SERIAL,
    goods_id integer, -- 类目ID
    image    text,    -- 商品图片
    goods_no text,    -- 产品编号 (暂时没有)
    sku_no   text,    -- sku编号
    color    text,    -- 颜色
    notes    text     -- 备注
);

insert into skus (goods_id, image, goods_no, sku_no, color, notes)
values
(1, 'http://baidu.com', 'goods_no_1', 'goods_1_sku_1', 'blue', 'notes....'),
(1, 'http://baidu.com', 'goods_no_1', 'goods_1_sku_2', 'blue', 'notes....'),
(1, 'http://baidu.com', 'goods_no_1', 'goods_1_sku_3', 'blue', 'notes....'),
(1, 'http://baidu.com', 'goods_no_1', 'goods_1_sku_4', 'blue', 'notes....'),
(1, 'http://baidu.com', 'goods_no_1', 'goods_1_sku_5', 'blue', 'notes....'),
(1, 'http://baidu.com', 'goods_no_1', 'goods_1_sku_6', 'blue', 'notes....'),
(2, 'http://baidu.com', 'goods_no_1', 'goods_2_sku_1', 'blue', 'notes....'),
(2, 'http://baidu.com', 'goods_no_1', 'goods_2_sku_2', 'blue', 'notes....'),
(2, 'http://baidu.com', 'goods_no_1', 'goods_2_sku_3', 'blue', 'notes....'),
(2, 'http://baidu.com', 'goods_no_1', 'goods_2_sku_4', 'blue', 'notes....'),
(2, 'http://baidu.com', 'goods_no_1', 'goods_2_sku_5', 'blue', 'notes....'),
(2, 'http://baidu.com', 'goods_no_1', 'goods_2_sku_6', 'blue', 'notes....');

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

-- 订单表
create table orders
(
    id            serial,
    customer_id   integer, -- 客户ID
    order_no      text,    -- 订单编号
    order_date    integer, -- 订货日期
    delivery_date integer  -- 交货日期
);

insert into orders (customer_id, order_no, order_date, delivery_date)
VALUES (1, 'order_no_1', 1691558739, 1691568739),
       (2, 'order_no_2', 1691558739, 1691568739),
       (3, 'order_no_3', 1691558739, 1691568739);

-- 订单商品表
create table order_items
(
    id               serial,
    order_id         integer, -- 订单ID
    sku_id           integer, -- 商品ID
    package_card     text,    -- 包装卡片    （存在大问题）
    package_card_des text,    -- 包装卡片说明 （存在大问题）
    count            integer, -- 数量
    unit             text,    --单位
    unit_price       integer, -- 单价
    total_price      integer, -- 总价/金额
    notes            text     -- 备注
);

-- insert into order_items (customer_id, order_no, order_date, delivery_date) VALUES (1, 'order_no_1', 1691558739, 1691568739);
-- insert into order_items (customer_id, order_no, order_date, delivery_date) VALUES (2, 'order_no_2', 1691558739, 1691568739);
-- insert into order_items (customer_id, order_no, order_date, delivery_date) VALUES (3, 'order_no_3', 1691558739, 1691568739);


-- 订单sku的的材料单 * N
create table order_item_materials
(
    id            serial,
    order_id      integer, -- 订单ID
    order_item_id integer, -- 订单商品ID
    name          text,    -- 材料名称
    color         text,    -- 材料颜色
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
create table department
(
    id            serial,
    department_id integer,
    name          text,   -- 部门名称
    index         integer -- 流程位续
);

-- 账号
create table account
(
    id            serial,
    name          text,
    account       text,
    password      text,
    department_id integer
);

-- create index idx_index on department (`index`);
