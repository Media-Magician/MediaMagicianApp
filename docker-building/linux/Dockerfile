FROM ubuntu:22.04 AS builder

# Set these depending on where these repo's are located!!
# APP_SRC should be connected to github:Media-Magician/MediaMagicianApp
# FFMPEG_SRC should be connected to github: Media-Magician/ffmpeg
ARG APP_SRC=./MediaMagicianApp
ARG FFMPEG_SRC=./ffmpeg
########################################

ENV DEBIAN_FRONTEND=noninteractive

# System dependencies
RUN apt-get update && apt-get install -y \
    libclang-dev \
    zlib1g-dev \
    xdg-utils \
    build-essential curl git pkg-config \
    nasm yasm libx264-dev \
    diffutils \
    autoconf automake libtool \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libjavascriptcoregtk-4.1-dev \
    libwebkit2gtk-4.1-dev \
    libsoup-3.0-dev \
    libgtk-3-dev \
    libglib2.0-dev \
    libbz2-dev liblzma-dev \
    libva-dev libvdpau-dev libdrm-dev libx11-dev \
    && rm -rf /var/lib/apt/lists/*
RUN apt-get remove -y libx264-dev libx264-163 || true

# Node 22
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Tauri CLI
RUN cargo install tauri-cli

# Copy sources
COPY ${FFMPEG_SRC} /build/ffmpeg
COPY ${APP_SRC} /build/app

# Add x264 source
RUN git clone https://code.videolan.org/videolan/x264.git /build/x264 && \
    cd /build/x264 && \
    git checkout stable && \
    ./configure --enable-shared --disable-static --disable-cli && \
    make -j$(nproc) && \
    make install && \
    ldconfig

RUN ls /usr/local/lib/libx264*

# Build FFmpeg
WORKDIR /build/ffmpeg
RUN autoreconf -fiv || true && \
    make distclean || true && \
    ./configure \
      --prefix=/build/app/src-tauri/assets/ubuntu/x86_64 \
      --arch=x86_64 \
      --disable-shared \
      --enable-static \
      --enable-small \
      --disable-programs \
      --disable-doc \
      --disable-debug \
      --disable-avdevice \
      --disable-postproc \
      --enable-protocol=file \
      --disable-network \
      --enable-libx264 \
      --enable-gpl \
      --extra-cflags="-fPIC" && \
    make clean && \
    make -j$(nproc) && \
    make install

# Build app
WORKDIR /build/app

# set env for build
ENV FFMPEG_ASSET_PATH=assets/ubuntu/x86_64
ENV FFMPEG_DIR=/build/app/src-tauri/assets/ubuntu/x86_64
ENV PKG_CONFIG_PATH=/build/app/src-tauri/assets/ubuntu/x86_64
ENV FFMPEG_STATIC="1"
ENV CARGO_FEATURE_STATIC="1"
ENV LIBRARY_PATH=/usr/local/lib:$LIBRARY_PATH
ENV LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
ENV RUSTFLAGS="-L /usr/local/lib"


RUN npm install
RUN cargo tauri build --debug

# Copy output binary
# RUN cp src-tauri/target/release/MediaMagician /MediaMagician
RUN mkdir /tmp/bundle && mkdir /tmp/bundle/deb && mkdir /tmp/bundle/rpm && \
    cp src-tauri/target/debug/MediaMagician /tmp/MediaMagician && \
    cp -r src-tauri/target/debug/bundle/deb/* /tmp/bundle/deb && \
    cp -r src-tauri/target/debug/bundle/rpm/* /tmp/bundle/rpm && \
    rm -rf src-tauri/target

FROM scratch
COPY --from=builder /tmp/MediaMagician /
COPY --from=builder /tmp/bundle /bundle
    