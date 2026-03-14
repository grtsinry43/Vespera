# Vespera LightMonitor

轻量级服务器监控系统，专为资源受限的 VPS 设计。

## 快速开始

### 使用 Docker Compose（推荐）

```bash
# 1. 克隆项目
git clone <repository-url>
cd Vespera

# 2. 构建并启动
docker-compose up -d

# 3. 查看日志
docker-compose logs -f
```

### 单独运行 Agent

```bash
# 使用环境变量
docker run -d \
  --name vespera-agent \
  -e VESPERA_NODE_ID=my-server \
  -e VESPERA_SERVER_URL=http://your-server:3000 \
  -e VESPERA_SECRET=your-secret \
  vespera-agent

# 或使用配置文件
docker run -d \
  --name vespera-agent \
  -v $(pwd)/agent.toml:/agent.toml \
  vespera-agent
```

### 本地开发

```bash
# 安装依赖
cargo build

# 准备 Server 环境
cp server/.env.example server/.env

# 初始化数据库并检查 Server
bash scripts/server-dev.sh check

# 初始化数据库并运行 Server
bash scripts/server-dev.sh run

# 运行 Agent
cp agent.toml.example agent.toml
# 编辑 agent.toml 配置
cargo run --bin vespera-agent
```

## 配置说明

### Agent 配置

创建 `agent.toml`：

```toml
[agent]
node_id = "my-server-01"
server_url = "http://localhost:3000"
report_interval = 5
timeout = 10
retry_attempts = 3

[auth]
secret = "your-secret-key"
```

或使用环境变量：
- `VESPERA_NODE_ID` - 节点 ID
- `VESPERA_SERVER_URL` - Server 地址
- `VESPERA_SECRET` - 认证密钥
- `VESPERA_REPORT_INTERVAL` - 上报间隔（秒）

### Server 配置

- `JWT_SECRET` - JWT 签名密钥，必填
- `AGENT_REGISTRATION_TOKEN` - Agent 上报认证密钥，必填
- `INITIAL_ADMIN_USERNAME` / `INITIAL_ADMIN_PASSWORD` - 首次启动创建管理员
- `SQLITE_PATH` 或 `DATABASE_URL` - SQLite 数据库位置
- `BIND_ADDRESS` 或 `SERVER_HOST` + `SERVER_PORT` - 监听地址

### Server 启动脚本

- `bash scripts/server-dev.sh bootstrap` - 初始化/修复本地开发数据库与 `_sqlx_migrations`
- `bash scripts/server-dev.sh check` - 用当前 `server/.env` 完成 DB bootstrap 并执行 `cargo check`
- `bash scripts/server-dev.sh run` - 用当前 `server/.env` 完成 DB bootstrap 并启动服务

## 性能指标

- **Agent 内存占用**: < 10MB
- **Agent CPU 占用**: < 1%
- **Server 内存占用**: < 50MB (支持 100 节点)
- **二进制大小**: Agent < 5MB, Server < 20MB
- **Docker 镜像**: Agent ~3-5MB, Server ~8-15MB

## 架构

```
┌─────────┐      ┌─────────┐      ┌─────────┐
│ Agent 1 │      │ Agent 2 │      │ Agent N │
└────┬────┘      └────┬────┘      └────┬────┘
     │                │                │
     └────────────────┼────────────────┘
                      │ HTTP/JSON
                 ┌────▼────┐
                 │  Server │
                 └────┬────┘
                      │
                 ┌────▼────┐
                 │  SQLite │
                 └─────────┘
```

## 开发

项目使用 Cargo Workspace 结构：

```
Vespera/
├── agent/          # Agent 二进制
├── server/         # Server 二进制
├── common/         # 共享库
├── Cargo.toml      # Workspace 配置
└── docker-compose.yml
```

### 编译优化

```bash
# Release 构建（优化二进制大小）
cargo build --release

# 检查二进制大小
ls -lh target/release/vespera-*
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定包的测试
cargo test -p vespera-agent
cargo test -p vespera-server
cargo test -p vespera-common
```

## License

MIT
