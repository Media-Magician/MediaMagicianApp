for f in /home/paige/Documents/GitHub/ceftauritest/buildplace/*; do
  highest=$(readelf -V "$f" 2>/dev/null | grep -o 'GLIBC_[0-9.]*' | sort -V | tail -1)
  if [ -n "$highest" ]; then
    echo "$highest  $f"
  fi
done