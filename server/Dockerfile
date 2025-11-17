# Server Dockerfile - 极致小镜像
# 多阶段构建：Rust 编译 + scratch 最小镜像

# ============ Build Stage ============
FROM rust:1.85-alpine AS builder

# 安装构建依赖
RUN apk add --no-cache musl-dev

# 设置工作目录
WORKDIR /build

# 复制整个项目（workspace）
COPY Cargo.toml Cargo.lock ./
COPY server ./server
COPY common ./common

# 构建 release 版本（静态链接）
RUN cargo build --release --bin vespera-server --target x86_64-unknown-linux-musl

# 验证二进制大小
RUN ls -lh /build/target/x86_64-unknown-linux-musl/release/vespera-server

# ============ Runtime Stage ============
FROM scratch

# 从 builder 复制二进制文件
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/vespera-server /vespera-server

# 创建数据目录（需要 volume 挂载）
# 注意：scratch 镜像无法创建目录，需要在运行时挂载

# 暴露端口
EXPOSE 3000

# 设置环境变量
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:///data/vespera.db

# 运行 server
ENTRYPOINT ["/vespera-server"]

# 预期镜像大小: 8-15MB
# 使用方式：
# docker run -p 3000:3000 \
#            -v $(pwd)/data:/data \
#            -e DATABASE_URL=sqlite:///data/vespera.db \
#            vespera-server
