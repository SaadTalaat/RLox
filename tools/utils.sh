
RED='\033[0;31m'
GREEN='\033[0;32m'
GREY='\033[0;37m'
NC='\033[0m'

msg() {
  echo "${GREY}$1${NC}"
}

error_msg() {
  echo "${RED}$1${NC}"
}

pass_msg() {
  echo "${GREEN}$1${NC}"
}

rlox_build() {
  msg ">> Building interpreter, profile = $PROFILE, target= $TARGET"
  err=$(cargo build --${PROFILE} --target=${TARGET} -p rlox 2>&1 >/dev/null)

  if [ $? != 0 ]; then
    echo "$err";
    error_msg "Failed to build interpreter.";
  fi
}

set_target() {
  target="$1"
  shift
  case "$target" in
    wasm )
      TARGET="wasm32-wasi"
      RUNTIME=$(command -v wasmer || command -v wasmtime || check_installed wasmtime)
      RUNTIME_OPTS="--dir=."
      RLOX="./target/${TARGET}/${PROFILE}/rlox.wasm"
      ;;
    * )
      arch=$(uname -m)
      TARGET=$(rustup target list --installed | grep "$arch")
      RLOX="./target/${TARGET}/${PROFILE}/rlox"
      ;;
  esac
}

check_installed() {
  cmd="$1"
  CMD=$(command -v ${cmd})
  if [ "$CMD" = "$cmd" ]; then
    error_msg "$cmd not installed"
    exit
  fi
}

