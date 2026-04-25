# Redis Sub Persistence

将 Redis Pub/Sub 订阅消息持久化到文件或数据库的工具。

## 功能特性

- 订阅 Redis 的多个 channel
- 将消息持久化到文件（支持数据库持久化，尚未完整实现）
- TOML 配置文件驱动
- 日志系统：支持控制台和文件双输出，支持多级别过滤
- 信号控制：支持 Ctrl-C / SIGTERM 优雅退出
- systemd 服务支持

## 编译

依赖 Rust 1.85+（edition 2024）。

```bash
# 开发版编译
cargo build

# 发布版编译（启用 LTO 优化）
cargo build --release
```

编译产物位于 `target/debug/` 或 `target/release/`。

## 配置

使用 TOML 格式的配置文件，默认路径为 `config.toml`。

### 完整配置示例

```toml
[redis]
host = "127.0.0.1"
port = 6379
db = 0
password = ""
channel = ["my-channel", "another-channel"]

[persistence]
type = "file"
file = "./data/messages.log"

# 以下字段仅在 type = "db" 时生效
# db_host = "127.0.0.1"
# db_port = 3306
# db_password = ""
# db_db = "redis_messages"
# db_timeout = 30
# db_retries = 3

[log]
level = "info"
file = "./logs/app.log"
```

### 配置项说明

#### `[redis]`

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `host` | string | 无（必填） | Redis 服务器地址 |
| `port` | u16 | 6379 | Redis 端口 |
| `db` | u64 | 0 | Redis 数据库编号 |
| `password` | string | "" | Redis 密码，为空表示无密码 |
| `channel` | string[] | 无（必填） | 要订阅的 channel 列表 |

#### `[persistence]`

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `type` | string | "file" | 持久化类型，可选 `file` 或 `db` |
| `file` | string | 无 | 文件持久化路径（type=file 时必填） |
| `db_host` | string | "127.0.0.1" | 数据库地址（type=db 时生效） |
| `db_port` | u16 | 3306 | 数据库端口 |
| `db_password` | string | 无 | 数据库密码 |
| `db_db` | string | "redis_messages" | 数据库名称 |
| `db_timeout` | u64 | 30 | 数据库连接超时（秒） |
| `db_retries` | u32 | 3 | 数据库重试次数 |

#### `[log]`

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `level` | string | "info" | 日志级别：`debug`、`info`、`warn`、`error` |
| `file` | string | 无 | 日志文件路径，不设置则仅输出到控制台 |

## 使用

### 前台运行

```bash
# 使用默认配置文件 config.toml
cargo run --release

# 指定配置文件路径
cargo run --release -- --config /path/to/my-config.toml

# 直接运行编译好的二进制
./target/release/redis-sub-persistence --config config.toml
```

### 验证运行

启动程序后，使用 `redis-cli` 发布测试消息：

```bash
redis-cli publish my-channel "hello world"
```

在持久化文件 `data/messages.log` 中可以看到：

```
[2026-04-25 16:27:46] channel=my-channel message=hello world
```

### 优雅退出

按下 `Ctrl-C` 或发送 `SIGTERM` 信号即可优雅停止程序：

```bash
kill -TERM <pid>
```

## 部署

### systemd 服务

项目附带 `redis-sub-persistence.service` 文件，用于 systemd 管理。

1. 编译发布版：

```bash
cargo build --release
```

2. 安装二进制和配置文件：

```bash
sudo cp target/release/redis-sub-persistence /usr/local/bin/
sudo mkdir -p /etc/redis-sub-persistence
sudo cp config.toml /etc/redis-sub-persistence/config.toml
```

3. 根据实际环境修改 `/etc/redis-sub-persistence/config.toml` 中的 Redis 地址、channel、持久化路径等配置。

4. 安装 systemd 服务文件：

```bash
sudo cp redis-sub-persistence.service /etc/systemd/system/
sudo systemctl daemon-reload
```

5. 启动和管理服务：

```bash
# 启动服务
sudo systemctl start redis-sub-persistence

# 设置开机自启
sudo systemctl enable redis-sub-persistence

# 查看运行状态
sudo systemctl status redis-sub-persistence

# 查看日志
sudo journalctl -u redis-sub-persistence -f

# 停止服务
sudo systemctl stop redis-sub-persistence

# 重启服务
sudo systemctl restart redis-sub-persistence
```

## 项目结构

```
src/
├── main.rs              # 入口：配置加载、初始化、启动订阅
├── config.rs            # TOML 配置解析
├── logger.rs            # 日志初始化（控制台 + 文件双输出）
├── subscriber.rs        # Redis Pub/Sub 异步订阅与消息接收
├── signal.rs            # 信号处理（Ctrl-C 优雅退出）
└── persistence/
    ├── mod.rs            # Persistence trait 与工厂方法
    ├── file.rs           # 文件持久化实现
    └── database.rs       # 数据库持久化（预留，尚未完整实现）
```

## 许可证

MIT