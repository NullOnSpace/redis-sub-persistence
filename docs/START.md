# Redis Sub Persistence
一个用于将redis的订阅消息持久化的工具

## 功能
- 订阅redis的channel
- 将订阅到的消息持久化到文件
- 支持toml配置文件

## 配置

### redis
- host: redis的host
- port: redis的port
- db: redis的db
- password: redis的password
- channel: 订阅的channel，列表格式，每个channel占一行

### persistence
- type: 持久化类型，支持file和db
- file: 持久化的文件路径
- db: 持久化的数据库路径
- db_port: 持久化的数据库端口
- db_password: 持久化的数据库密码
- db_db: 持久化的数据库db
- db_timeout: 持久化的数据库超时时间
- db_retries: 持久化的数据库重试次数

### 其他
- log_level: 日志级别，支持debug、info、warn、error、fatal
- log_file: 日志文件路径

## 日志系统
- 支持日志级别
- 支持日志文件
- 支持日志控制台输出

## 后台运行
支持在systemd中运行，支持信号量控制, 支持重启，支持停止

为systemd添加服务文件生成示例配置，通过以下命令复制到/systemd/system目录
```
cp redis-sub-persistence.service /systemd/system/
```
