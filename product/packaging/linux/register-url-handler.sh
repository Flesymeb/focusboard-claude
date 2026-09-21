#!/usr/bin/env bash
# Opt-in focusboard:// URL-handler registration for Focusboard on Linux.
#
# This script is measurement and deployment infrastructure, not product
# behavior: nothing in the application invokes it, no startup path registers
# anything, and running it never touches Focusboard profiles, sessions, or
# data. The Host cold-activation probe calls it explicitly, captures the
# emitted before/after handler state, and can undo it with --unregister.
#
# Usage:
#   register-url-handler.sh register    [--binary PATH]   install the mapping
#   register-url-handler.sh unregister                    remove the mapping
#   register-url-handler.sh status                        print current state
#
# Output: one "key=value" line per fact, ending with a single JSON line:
#   {"event":"focusboard_url_handler",...,"before":...,"after":...}
# Exit 0 on success, nonzero with state still emitted on failure.

set -u

scheme="focusboard"
desktop_id="focusboard.desktop"
script_dir="$(cd "$(dirname "$0")" && pwd)"
source_desktop="$script_dir/$desktop_id"
applications_dir="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
installed_desktop="$applications_dir/$desktop_id"

emit_json() {
    # before/after: null, "none", or the mapped desktop entry id
    printf '{"event":"focusboard_url_handler","scheme":"%s","action":"%s","before":%s,"after":%s,"desktop_entry":%s,"status":"%s"}\n' \
        "$scheme" "$action" "$before_json" "$after_json" "$entry_json" "$status"
}

to_json_string() {
    if [ "$1" = "none" ] || [ -z "$1" ]; then
        printf 'null'
    else
        printf '"%s"' "$1"
    fi
}

query_handler() {
    if command -v xdg-mime >/dev/null 2>&1; then
        xdg-mime query default "x-scheme-handler/$scheme" 2>/dev/null || true
    fi
}

action="${1:-}"
case "$action" in
    register|unregister|status) ;;
    *)
        echo "usage: $0 {register|unregister|status} [--binary PATH]" >&2
        exit 2
        ;;
esac
shift

binary=""
while [ $# -gt 0 ]; do
    case "$1" in
        --binary)
            [ $# -ge 2 ] || { echo "--binary requires a path" >&2; exit 2; }
            binary="$2"
            shift 2
            ;;
        *) echo "unknown option: $1" >&2; exit 2 ;;
    esac
done

before="$(query_handler)"
[ -n "$before" ] || before="none"
before_json="$(to_json_string "$before")"
echo "handler_before=$before"

if [ "$action" = "status" ]; then
    after="$before"
    after_json="$before_json"
    entry_json="$(to_json_string "$installed_desktop")"
    echo "handler_after=$after"
    echo "desktop_entry=$installed_desktop"
    echo "status=ok"
    status="ok"
    emit_json
    exit 0
fi

if [ "$action" = "unregister" ]; then
    if [ -f "$installed_desktop" ]; then
        rm -f "$installed_desktop"
    fi
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$applications_dir" 2>/dev/null || true
    fi
    after="$(query_handler)"
    [ -n "$after" ] || after="none"
    after_json="$(to_json_string "$after")"
    entry_json="$(to_json_string "$installed_desktop")"
    echo "handler_after=$after"
    echo "desktop_entry=$installed_desktop"
    echo "status=unregistered"
    status="unregistered"
    emit_json
    exit 0
fi

# register
if [ ! -f "$source_desktop" ]; then
    echo "error: desktop entry not found: $source_desktop" >&2
    after="$before"
    after_json="$before_json"
    entry_json="null"
    status="desktop_entry_missing"
    echo "handler_after=$after"
    echo "status=$status"
    emit_json
    exit 1
fi

if [ -z "$binary" ]; then
    for candidate in focusboard Focusboard; do
        if command -v "$candidate" >/dev/null 2>&1; then
            binary="$(command -v "$candidate")"
            break
        fi
    done
fi
if [ -z "$binary" ]; then
    echo "error: Focusboard executable not found; pass --binary PATH" >&2
    after="$before"
    after_json="$before_json"
    entry_json="null"
    status="binary_not_found"
    echo "handler_after=$after"
    echo "status=$status"
    emit_json
    exit 1
fi

mkdir -p "$applications_dir"
# Bind the installed entry to the resolved binary so the mapping is candidate-
# specific; %u hands the focusboard:// URL to the launched instance.
sed "s|^Exec=.*|Exec=$binary %u|" "$source_desktop" > "$installed_desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$applications_dir" 2>/dev/null || true
fi

if command -v xdg-mime >/dev/null 2>&1; then
    xdg-mime default "$desktop_id" "x-scheme-handler/$scheme" || {
        after="$before"
        after_json="$before_json"
        entry_json="$(to_json_string "$installed_desktop")"
        status="xdg_mime_default_failed"
        echo "handler_after=$after"
        echo "status=$status"
        emit_json
        exit 1
    }
else
    echo "error: xdg-mime unavailable; cannot register handler" >&2
    after="$before"
    after_json="$before_json"
    entry_json="$(to_json_string "$installed_desktop")"
    status="xdg_mime_missing"
    echo "handler_after=$after"
    echo "status=$status"
    emit_json
    exit 1
fi

after="$(query_handler)"
[ -n "$after" ] || after="none"
after_json="$(to_json_string "$after")"
entry_json="$(to_json_string "$installed_desktop")"
echo "handler_after=$after"
echo "desktop_entry=$installed_desktop"
echo "exec=$binary %u"
echo "status=registered"
status="registered"
emit_json

if [ "$after" = "none" ]; then
    echo "error: handler query still empty after registration" >&2
    exit 1
fi
exit 0
