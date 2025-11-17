# Agent Dockerfile - 极致小镜像
# 多阶段构建:Rust 编译 + scratch 最小镜像

# ============ Build Stage ============
FROM rust:1.85-alpine AS builder

# 安装构建依赖 (musl-dev for static linking)
RUN apk add --no-cache musl-dev

# 添加 musl target
RUN rustup target add x86_64-unknown-linux-musl

# 设置工作目录
WORKDIR /build

# 复制 workspace 配置
COPY Cargo.toml Cargo.lock ./

# 复制所有 workspace 成员的 Cargo.toml (Cargo 需要验证所有成员)
COPY common/Cargo.toml ./common/
COPY agent/Cargo.toml ./agent/
COPY server/Cargo.toml ./server/

# 复制 common 和 agent 的源代码 (只需要这两个)
COPY common/src ./common/src
COPY agent/src ./agent/src

# 为 server 创建空的 src 目录和 lib.rs (满足 Cargo workspace 验证)
RUN mkdir -p ./server/src && echo "fn main() {}" > ./server/src/main.rs

# 构建 release 版本 (静态链接 musl)
RUN cargo build --release --bin vespera-agent --target x86_64-unknown-linux-musl

# 验证二进制大小
RUN ls -lh /build/target/x86_64-unknown-linux-musl/release/vespera-agent

# ============ Runtime Stage ============
FROM scratch

# 从 builder 复制二进制文件
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/vespera-agent /vespera-agent

# 设置环境变量(可被 docker run -e 覆盖)
ENV VESPERA_SERVER_URL=http://server:3000
ENV VESPERA_REPORT_INTERVAL=5
ENV RUST_LOG=info

# 运行 agent
ENTRYPOINT ["/vespera-agent"]

# 预期镜像大小: 3-5MB
# 使用方式:
# docker run -e VESPERA_NODE_ID=my-node \
#            -e VESPERA_SERVER_URL=http://your-server:3000 \
#            -e VESPERA_SECRET=your-secret \
#            vespera-agent
