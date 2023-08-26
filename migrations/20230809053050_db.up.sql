-- 商品表
create table goods
(
    id       SERIAL,
    goods_no text not null default '', -- 类目编号
    image    text not null default '', -- 图片
    name     text not null default '', -- 名称
    notes    text                      -- 备注
);
create unique index uniq_goods_goods_no on goods (goods_no);

insert into goods (goods_no, image, name, notes)
values ('goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_1',
        'notes_1'),
       ('goods_no_2', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_2',
        'notes_2'),
       ('goods_no_3', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_3',
        'notes_3'),
       ('goods_no_4', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_4',
        'notes_4'),
       ('goods_no_5', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'goods_5',
        'notes_5');

-- sku表
create table skus
(
    id       SERIAL,                   -- ID
    goods_no text not null default '', -- 类目编号
    image    text,                     -- 商品图片  // todo: 感觉可有可无
    plating  text not null default '', -- 电镀
    color    text not null default '', -- 颜色
    notes    text                      -- 备注
);
create unique index uniq_skus_goods_no_and_color on skus (goods_no, color);


insert into skus (goods_no, image, plating, color, notes)
values ('goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'plating', '绿色',
        'notes....');
insert into skus (goods_no, image, plating, color, notes)
values ('goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'plating', '红色',
        'notes....');
insert into skus (goods_no, image, plating, color, notes)
values ('goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'plating', '粉红',
        'notes....');
insert into skus (goods_no, image, plating, color, notes)
values ('goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'plating', '白色',
        'notes....');
insert into skus (goods_no, image, plating, color, notes)
values ('goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'plating', '黑色',
        'notes....');
insert into skus (goods_no, image, plating, color, notes)
values ('goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'plating', '黄色',
        'notes....');
insert into skus (goods_no, image, plating, color, notes)
values ('goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'plating', '橙色',
        'notes....');
insert into skus (goods_no, image, plating, color, notes)
values ('goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'plating', '青色',
        'notes....');
insert into skus (goods_no, image, plating, color, notes)
values ('goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png', 'plating', '紫色',
        'notes....');

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
create unique index uniq_customers_customer_no on customers (customer_no);

insert into customers (customer_no, name, address, phone, notes)
values ('L1001', '', '', '', '');
insert into customers (customer_no, name, address, phone, notes)
values ('L1003', '', '', '', '');
insert into customers (customer_no, name, address, phone, notes)
values ('L1004', '', '', '', '');
insert into customers (customer_no, name, address, phone, notes)
values ('L1005', '', '', '', '');
insert into customers (customer_no, name, address, phone, notes)
values ('L1006', '', '', '', '');
insert into customers (customer_no, name, address, phone, notes)
values ('L1007', '', '', '', '');

create table customer_excel_template(
    id serial,
--    customer_id integer not null,
    customer_no text not null,
    template_id integer not null
);

insert into customer_excel_template (customer_no, template_id)
values ('L1001', 1);
insert into customer_excel_template (customer_no, template_id)
values ('L1003', 1);
insert into customer_excel_template (customer_no, template_id)
values ('L1006', 1);
-- 虽然和上面不太一样

insert into customer_excel_template (customer_no, template_id)
values ('L1005', 1);

insert into customer_excel_template (customer_no, template_id)
values ('L1012', 2);



-- 订单表
create table orders
(
    id              serial,
    customer_id     integer not null,               -- 客户ID
    order_no        text    not null,               -- 订单编号
    order_date      integer not null,               -- 订货日期
    delivery_date   integer,                        -- 交货日期
    is_urgent       boolean not null default false, -- 加急
    is_return_order boolean not null default false  -- 返单
);

create unique index uniq_orders_order_no on orders (order_no);

insert into orders (customer_id, order_no, order_date, delivery_date, is_urgent, is_return_order)
VALUES (1, 'order_no_1', 1691558739, 1691568739, false, false),
       (2, 'order_no_2', 1691558739, 1691568739, true, false),
       (3, 'order_no_3', 1691558739, 1691568739, true, true);

-- 订单商品表
create table order_goods
(
    id               serial,
    order_id         integer not null, -- 订单ID
    order_no         text    not null, -- 订单编号
    goods_no         text    not null, -- 商品编号
    package_card     text,             -- 标签图片
    package_card_des text              -- 标签说明
);
create index idx_order_goods_order_id on order_goods (order_id);
create unique index uniq_order_goods_order_no_and_goods_no on order_goods (order_no, goods_no);

insert into order_goods (order_id, order_no, goods_no, package_card, package_card_des)
VALUES (1, 'order_no_1', 'goods_no_1', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png',
        '');
insert into order_goods (order_id, order_no, goods_no, package_card, package_card_des)
values (1, 'order_no_1', 'goods_no_2', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png',
        '');
insert into order_goods (order_id, order_no, goods_no, package_card, package_card_des)
values (1, 'order_no_1', 'goods_no_3', 'https://zos.alipayobjects.com/rmsportal/jkjgkEfvpUPVyRjUImniVslZfWPnJuuZ.png',
        '');

-- 订单sku表
create table order_items
(
    id          serial,
    order_id    integer not null, -- 订单ID
    order_no    text    not null, -- 商品编号
    sku_id      integer not null, -- sku id
    count       integer not null, -- 数量
    unit        text,             -- 单位
    unit_price  integer,          -- 单价
    total_price integer,          -- 总价/金额
    notes       text              -- 备注
);
create index idx_order_items_order_id on order_items (order_id);
create unique index uniq_order_goods_order_no_and_sku_id on order_items (order_no, sku_id);

insert into order_items (order_id, order_no, sku_id, count, unit)
values (1, 'order_no_1', 1, 10, '个');
insert into order_items (order_id, order_no, sku_id, count, unit)
values (1, 'order_no_1', 2, 10, '个');
insert into order_items (order_id, order_no, sku_id, count, unit)
values (1, 'order_no_1', 3, 10, '个');

-- 订单sku的的材料单 * N
create table order_item_materials
(
    id            serial,
    order_id      integer,       -- 订单ID
    order_item_id integer,       -- 订单商品ID
    name          text not null, -- 材料名称
    color         text not null, -- 材料颜色
--     material_id   integer, -- 材料ID  (暂时先不用)
    single        integer,       -- 单数      ？小数
    count         integer,       -- 数量      ？小数
    total         integer,       -- 总数(米)  ? 小数
    stock         integer,       -- 库存 ?
    debt          integer,       -- 欠数
    notes         text           -- 备注
);


-- 部门
create table departments
(
    id            serial,
--    department_id integer,
    name          text,   -- 部门名称
    index         integer -- 流程位续
);

insert into departments (name, index)
values ('业务部', 0);

-- 账号
create table accounts
(
    id            serial,
    name          text,
    account       text,
    password      text,
    department_id integer
);
insert into accounts (name, account, password, department_id)
values ('test', 'test', 'test', 1);

-- create index idx_index on department (`index`);
