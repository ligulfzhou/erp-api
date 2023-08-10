-- 供应商表
create table suppliers
(
    id          serial,
    supplier_no text, -- 供应商编号
    name        text, -- 名称
    notes       text  -- 备注
);

-- 订单表
create table orders
(
    id            serial,
    suppler       text,    -- 客户
    supplier_id   integer, -- 客户ID
    order_no      text,    -- 订单编号
    order_date    integer, -- 订货日期
    delivery_date integer  -- 交货日期
);

-- 订单商品表
create table order_items
(
    id               serial,
    order_id         integer, -- 订单ID
    item_id          integer, -- 商品ID
    package_card     text,    -- 包装卡片
    package_card_des text,    -- 包装卡片说明
    image            text     -- 商品图片
);

-- 订单商品的一个颜色
create table order_item_colors
(
    id            serial,
    order_item_id integer, -- 订单的商品ID
    order_id      integer, -- 订单ID
    color         text,    -- 颜色
    count         integer, -- 数量
    unit          text,    -- 单位
    unit_price    integer, -- 单价
    sum_of_money  integer, -- 金额
    notes         text     -- 备注
);

-- 订单商品的一个颜色的材料单
create table order_item_color_materials
(
    id          serial,
    name        text,    -- 材料名称
    color       text,    -- 材料颜色
    material_id integer, -- 材料ID (暂时先不用)
    single      integer, -- 单数      ？小数
    count       integer, -- 数量      ？小数
    total       integer, -- 总数(米)  ? 小数
    stock       integer, -- 库存 ?
    debt        integer, --欠数
    notes       text     -- 备注
);

create table item
(
    id         SERIAL,
    image      text,
    name       text,
    plating    text,
    num        integer,
    size       text,
    buy_price  integer,
    sell_price integer,
    memo       text
);

-- 材料表
create table material
(
    id    serial,
    name  text, -- 材料名
    color text, -- 材料颜色
    info  text  -- 材料说明
);

-- todo: 库存何时减少？ 在仓库确认之后？
create table material_stock
(
    id          serial,
    material_id integer,
    buy_price   integer,
    count       integer
);

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
