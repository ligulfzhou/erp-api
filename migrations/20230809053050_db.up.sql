-- 商品表
create table goods
(
    id               SERIAL,
    customer_no      text not null default '',     -- 客户ID
    goods_no         text not null default '',     -- 类目编号(可以为空，主要来自L1005)
    images           text[] not null default '{}', -- 图片 TODO: 只记录最后出现的图片
    image_des        text not null default '',     -- 图片
    name             text not null default '',     -- 名称
    plating          text not null default '',     -- 电镀
    package_card     text not null default '',     -- 标签图片
    package_card_des text not null default '',     -- 标签说明
    notes            text not null default ''      -- 备注
);
create index idx_goods_customer_no on goods (customer_no);
create unique index uniq_goods_goods_no on goods (goods_no);

-- sku表
create table skus
(
    id       SERIAL,                      -- ID
    goods_id integer not null default 0,  -- 类目ID
    sku_no   text    not null default '', -- SKU ID
    color    text    not null default '', -- 颜色
    color2   text    not null default '', -- 颜色（只记录，只会用上面的color）
    notes    text    not null default ''  -- 备注
);
create unique index uniq_skus_goods_id_and_color on skus (goods_id, color);

-- 客户
create table customers
(
    id          serial,
    customer_no text not null default '', -- 客户编号
    name        text not null default '', -- 名称
    address     text not null default '', -- 地址
    phone       text not null default '', -- 电话
    notes       text not null default ''  -- 备注
);
create unique index uniq_customers_customer_no on customers (customer_no);

insert into customers (customer_no, name, address, phone, notes)
values ('L1001', '', '', '', '');
insert into customers (customer_no, name, address, phone, notes)
values ('L1002', '', '', '', '');
insert into customers (customer_no, name, address, phone, notes)
values ('L1003', '', '', '', '');
insert into customers (customer_no, name, address, phone, notes)
values ('L1004', '', '', '', '');
insert into customers (customer_no, name, address, phone, notes)
values ('L1005', '', '', '', '');
insert into customers (customer_no, name, address, phone, notes)
values ('L1006', '', '', '', '');
-- insert into customers (customer_no, name, address, phone, notes)
-- values ('L1007', '', '', '', '');
-- insert into customers (customer_no, name, address, phone, notes)
-- values ('L1012', '', '', '', '');

create table customer_excel_template
(
    id          serial,
    customer_no text    not null,
    template_id integer not null
);

insert into customer_excel_template (customer_no, template_id)
values ('L1001', 1);
insert into customer_excel_template (customer_no, template_id)
values ('L1002', 1);
insert into customer_excel_template (customer_no, template_id)
values ('L1003', 1);
insert into customer_excel_template (customer_no, template_id)
values ('L1006', 1);

-- 虽然和上面不太一样
insert into customer_excel_template (customer_no, template_id)
values ('L1005', 2);

insert into customer_excel_template (customer_no, template_id)
values ('L1012', 3);

insert into customer_excel_template (customer_no, template_id)
values ('L1004', 4);

-- 订单表
create table orders
(
    id               serial,
    customer_no      text    not null,               -- 客户ID
    order_no         text    not null,               -- 订单编号
    order_date       date    not null,               -- 订货日期
    delivery_date    date,                           -- 交货日期
    is_urgent        boolean not null default false, -- 加急
    is_return_order  boolean not null default false, -- 返单
    is_special       boolean not null default false, -- 特别客人订单
    special_customer text    not null default ''
);
create unique index uniq_orders_order_no on orders (order_no);
create index idx_orders_customer_no_order_date on orders (customer_no, order_date);

-- 订单商品表
create table order_goods
(
    id               serial,
    index            integer not null default 0,   -- 用于排序
    images           text[] not null default '{}', -- 图片 TODO: 只记录最后出现的图片
    image_des        text    not null default '',  -- 图片
    package_card     text    not null default '',  -- 标签图片
    package_card_des text    not null default '',  -- 标签说明
    order_id         integer not null,             -- 订单ID
    goods_id         integer not null              -- 商品ID
);
create index idx_order_goods_order_id on order_goods (order_id);
create unique index uniq_order_goods_order_id_and_goods_id on order_goods (order_id, goods_id);

-- 订单sku表
create table order_items
(
    id             serial,
    order_goods_id integer not null,             -- 订单商品ID
    order_id       integer not null,             -- 订单ID
    sku_id         integer not null,             -- sku id
    count          integer not null,             -- 数量
    unit           text,                         -- 单位
    unit_price     integer,                      -- 单价
    total_price    integer,                      -- 总价/金额
    notes_images   text[] not null default '{}', -- 备注内的图片
    notes          text    not null default ''   -- 备注
);
-- goods_id       integer not null, -- 商品ID
-- create index idx_order_items_goods_id on order_items (goods_id);
create index idx_order_items_order_id on order_items (order_id);
create index idx_order_items_sku_id on order_items (sku_id);
create unique index uniq_order_items_order_id_and_sku_id on order_items (order_id, sku_id);

create table progress
(
    id            serial,
    order_item_id integer     not null default 0,
    step          integer     not null default 0, -- 哪一步
    index         integer     not null default 0,
    account_id    integer     not null default 0,
    done          boolean     not null default false,
    notes         text        not null default '',
    dt            timestamptz not null
);

-- -- 订单sku的的材料单 * N
-- create table order_item_materials
-- (
--     id            serial,
--     order_id      integer,       -- 订单ID
--     order_item_id integer,       -- 订单商品ID
--     name          text not null, -- 材料名称
--     color         text not null, -- 材料颜色
--     single        integer,       -- 单数      ？小数
--     count         integer,       -- 数量      ？小数
--     total         integer,       -- 总数(米)  ? 小数
--     stock         integer,       -- 库存 ?
--     debt          integer,       -- 欠数
--     notes         text           -- 备注
-- );

-- 部门
create table departments
(
    id    serial,
    name  text not null default '',       -- 部门名称
    steps integer[] not null default '{}' -- 流程位续
);

insert into departments (name, steps)
values ('业务部', '{1}');
insert into departments (name, steps)
values ('仓库部', '{2}');

-- 账号
create table accounts
(
    id            serial,
    name          text    not null default '',
    account       text    not null default '',
    password      text    not null default '',
    department_id integer not null default 0
);
insert into accounts (name, account, password, department_id)
values ('业务test', 'test', 'test', 1);
insert into accounts (name, account, password, department_id)
values ('业务小白', 'yewuxiaobai', 'yewuxiaobai', 1);
insert into accounts (name, account, password, department_id)
values ('仓库小黄', 'cangkuxiaohuang', 'cangkuxiaohuang', 2);
