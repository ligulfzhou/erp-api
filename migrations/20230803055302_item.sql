-- Add migration script here

create table item (
    id SERIAL,
    num integer,
    size text,
    buy_price integer,
    sell_price integer,
    memo text
);
