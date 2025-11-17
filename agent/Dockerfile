# Agent Dockerfile - 极致小镜像
# 多阶段构建：Rust 编译 + scratch 最小镜像

# ============ Build Stage ============
FROM rust:1.85-alpine AS builder

# 安装构建依赖
RUN apk add --no-cache musl-dev

# 设置工作目录
WORKDIR /build

# 复制整个项目（workspace）
COPY Cargo.toml Cargo.lock ./
COPY agent ./agent
COPY common ./common

# 构建 release 版本（静态链接）
RUN cargo build --release --bin vespera-agent --target x86_64-unknown-linux-musl

# 验证二进制大小
RUN ls -lh /build/target/x86_64-unknown-linux-musl/release/vespera-agent

# ============ Runtime Stage ============
FROM scratch

# 从 builder 复制二进制文件
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/vespera-agent /vespera-agent

# 设置环境变量（可被 docker run -e 覆盖）
ENV VESPERA_SERVER_URL=http://server:3000
ENV VESPERA_REPORT_INTERVAL=5
ENV RUST_LOG=info

# 运行 agent
ENTRYPOINT ["/vespera-agent"]

# 预期镜像大小: 3-5MB
# 使用方式：
# docker run -e VESPERA_NODE_ID=my-node \
#            -e VESPERA_SERVER_URL=http://your-server:3000 \
#            -e VESPERA_SECRET=your-secret \
#            vespera-agent
