
benchmark_print_usage() {
  echo "Usage: invoke benchmark [OPTIONS]"
  echo
  echo "-r:                 test using release build"
  echo "-d:                 test using debug build"
  echo "-t:                 target (e.g. bin, wasm)"
  echo "-q:                 saves logs to a file"
  echo "-h:                 prints this message"
  echo
  echo "Examples:"
  echo "invoke benchmark -rt x86_64"
}

run_benchmarks() {
  msg ">> Running benchmarks"
  for tst in tests/benchmarks/*.lox; do
    rlox="$RUNTIME $RUNTIME_OPTS $RLOX"
    bold_msg "$(head -n1 "$tst" |sed 's/^.\{3\}//')"
    $rlox "$tst"
  done;
}

rlox_benchmark() {
  set_target "default"
  while getopts ":t:hrdq" option; do
    case "$option" in
      h)
        benchmark_print_usage
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
        # RESET target
        set_target "$TARGET"
        ;;
      t)
        set_target "$OPTARG"
        msg ">> Set target = $TARGET"
        if [ "$RUNTIME" != "" ]; then
          msg ">> Set runtime = $RUNTIME"
        fi
        ;;

      q)
        OUTFILE=$(${DATE} +"%d-%h-%y_%H-%M-%S.log")
        ;;

      \?)
        break
        ;;
    esac
  done

  rlox_build
  if [ "$OUTFILE" = "" ]; then
    run_benchmarks
  else
    mkdir -p logs/benchmarks/"$TARGET/$PROFILE_PATH"
    OUTFILE="logs/benchmarks/$TARGET/$PROFILE_PATH/$OUTFILE"
    msg "Saving output to, $OUTFILE ..."
    run_benchmarks > "$OUTFILE"
  fi
  bold_msg "$OUTFILE"
}
