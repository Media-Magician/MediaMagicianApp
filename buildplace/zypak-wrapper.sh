#!/bin/bash

export ZYPAK_ZYGOTE_STRATEGY=direct
export LD_PRELOAD=""
export ZYPAK_SANDBOX_STRATEGY=noop
export CHROME_DEVEL_SANDBOX=/app/bin/chrome-sandbox
export ZYPAK_ZYGOTE_STRATEGY_DIRECT_SPAWN_STRATEGY=1
export GTK_USE_PORTAL=0
export TMPDIR=$XDG_RUNTIME_DIR/app/$FLATPAK_ID
# exec zypak-wrapper /app/bin/mediamagician --no-sandbox "$@"
# exec zypak-wrapper /app/bin/mediamagician \
exec /app/bin/mediamagician \
  --no-sandbox \
  --no-zygote \
  --locales-dir-path=/app/bin/locales \
  "$@"
