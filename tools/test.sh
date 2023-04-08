
load_module() {
  module="$1"
  . "$module" || {
    echo "Couldn't load module $module"
      exit 1
    }
}

modules=(
  tools/utils.sh
)
for module in "${modules[@]}"; do
  load_module $module
done

tests_print_usage() {
  echo "Usage: invoke test <cmd> [OPTIONS]"
  echo
  echo "-r:                 test using release build"
  echo "-d:                 test using debug build"
  echo "-t:                 target (e.g. bin, wasm)"
  echo "-h:                 prints this message"
  echo
  echo "Examples:"
  echo "invoke test all -rt x86_64"
  echo
  echo "Commands:"
  echo "lexical             runs lexical analysistests"
  echo "syntax              runs syntax analysis tests"
  echo "resolution          runs semantic analysis tests"
  echo "runtime             runs runtime behavior tests"
  echo "all                 runs all tests"
}

run_tests() {
  test_name=$1
  test_dirname=$2
  test_errno=$3
  msg ">> Generating [$test_name] tests @ $test_dirname/all"
  python3 tools/generate.py "$test_dirname/all"
  passed=0
  failures=0

  for tst in "$test_dirname"/*.lox; do
    rlox="$RUNTIME $RUNTIME_OPTS $RLOX "
    stderr=$($rlox "$tst" 2>&1 >/dev/null)
    errno=$?
    if [ "$errno" != "$test_errno" ]; then
      error_msg "$tst [FAILED] (errno: $errno)"
      msg "$stderr"
      failures=$((failures + 1))
    else
      pass_msg "$tst [OK]"
      passed=$((passed + 1))
    fi
  done
  msg ">> Ran $passed tests passed, $failures failed."
  rm "$test_dirname"/*.lox
  return $failures
}


rlox_test() {
  then=$($DATE +"%s%N")
  failed=0
  cmd=$1
  shift
  set_target "default"
  while getopts ":t:hrd" option; do
    case "$option" in
      h)
        tests_print_usage
        exit
        ;;
      r )
        msg ">> Using release profile"
        PROFILE=release
        PROFILE_PATH=release
        set_target "$TARGET"
        ;;
      d )
        msg ">> Using debug profile"
        PROFILE=dev
        PROFILE_PATH=debug
        set_target "$TARGET"
        ;;
      t)
        set_target "$OPTARG"
        msg ">> Set target = $TARGET"
        if [ "$RUNTIME" != "" ]; then
          msg ">> Set runtime = $RUNTIME"
        fi
        ;;
      \?)
        break
        ;;
    esac
  done

  shift
  case $cmd in
    lexical )
      rlox_build
      run_tests "lexical" tests/lex 101
      [ $? -eq 0 ] || failed=1
      ;;
    syntax )
      rlox_build
      run_tests "syntax" tests/parse 102
      [ $? -eq 0 ] || failed=1
      ;;
    resolution )
      rlox_build
      run_tests "resolution" tests/resolve 103
      [ $? -eq 0 ] || failed=1
      ;;
    runtime )
      rlox_build
      run_tests "runtime" tests/runtime 104
      [ $? -eq 0 ] || failed=1
      ;;
    all )
      rlox_build
      run_tests "lexical" tests/lex 101
      [ $? -eq 0 ] || failed=1
      run_tests "syntax" tests/parse 102
      [ $? -eq 0 ] || failed=1
      run_tests "resolution" tests/resolve 103
      [ $? -eq 0 ] || failed=1
      run_tests "runtime" tests/runtime 104
      [ $? -eq 0 ] || failed=1
      ;;
    * )
      tests_print_usage
      exit 1
      ;;
  esac
  now=$($DATE +"%s%N")
  elapsed=$(( (now - then) / 1000000))
  echo ">> Finished in $GREEN($elapsed ms).$NC"
  [ $failed -eq 0 ] || exit 1
}
