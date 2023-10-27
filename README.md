# erp-api

## 如何运行
```
1: 安装postgresql, 创建erp数据库，并修改.env文件里的连续数据库用的账号信息
2: sqlx mig run（创建表结构）
3: 安装rust，并执行 cargo watch -q -c -w  src/ -x run
```


```javascript

export const progresses: StepProgress[] = [
    {
        step: 1,
        name: "下订单",
        department: "业务部",
        options: [
            {
                index: 3,
                name: "颜色打样",
                color: "#FFFE00"
            },
            {
                index: 4,
                name: "成品不锈钢订货",
                color: "#92D04F",
            },
            {
                index: 5,
                name: "打材料单",
                color: "#00B0F0"
            },
            {
                index: 1,
                name: "异常(备注)",
                color: "#C9D2DC"
            },
            {
                index: 2,
                name: "已完成",
                color: "#C9D2DC"
            }
        ]
    },
    {
        step: 2,
        name: "仓库",
        department: "仓库部",
        options: [
            {
                index: 3,
                name:"已发车间",
                color: "#7030A1"
            },
            {
                index: 4,
                name:"已发品检",
                color: "#02B151",
            },
            {
                index: 5,
                name:"已发滴油",
                color: "#FFC100",
            },
            {
                index: 6,
                name: "兰溪JLZ",
                color: "#FF0000"
            },
            {
                index: 7,
                name: "兰溪FW",
                color: "#BF0101",
            },
            {
                index: 8,
                name: "兰溪LSS",
                color: "#D9E4BC"
            },
            {
                index: 9,
                name: "兰溪CN",
                color: "#8DB3E2"
            },
            {
                index: 1,
                name: "异常(备注)",
                color: "#C9D2DC"
            },
            {
                index: 2,
                name: "已完成",
                color: "#C9D2DC"
            }
        ]
    },
    {
        step: 3,
        name: "生产",
        department: "生产部",
        options: [
            {
                index: 3,
                name: "外发",
                color: "#CCC1DB"
            },
            {
                index: 4,
                name: "生产",
                color: "#528DD5",
            },
            {
                index: 1,
                name: "异常(备注)",
                color: "#C9D2DC"
            },
            {
                index: 2,
                name: "已完成",
                color: "#C9D2DC"
            }
        ]
    },
    {
        step: 4,
        name: "品检",
        department: "品检部",
        options: [
            {
                index: 1,
                name: "异常(备注)",
                color: "#C9D2DC"
            },
            {
                index: 2,
                name: "已完成",
                color: "#C9D2DC"
            }
        ]
    },
    {
        step: 5,
        name: "包装",
        department: "包装部",
        options: [
            {
                index: 3,
                name: "兰溪包装",
                color: "#FAC08F"
            },
            {
                index: 4,
                name: "义乌包装",
                color: "#E8B7B8",
            },
            {
                index: 1,
                name: "异常(备注)",
                color: "#C9D2DC"
            },
            {
                index: 2,
                name: "已完成",
                color: "#C9D2DC"
            }
        ]
    },
    {
        step: 6,
        name: "出货",
        department: "装箱部",
        options: [
            {
                index: 1,
                name: "异常(备注)",
                color: "#C9D2DC"
            },
            {
                index: 2,
                name: "已完成",
                color: "#C9D2DC"
            }
        ]
    },
    {
        step: 7,
        name: "拍照",
        department: "业务部",
        options: [
            {
                index: 1,
                name: "异常(备注)",
                color: "#C9D2DC"
            },
            {
                index: 2,
                name: "已完成",
                color: "#C9D2DC"
            }
        ]
    },
]
```