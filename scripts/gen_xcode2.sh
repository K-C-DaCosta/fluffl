#!/bin/bash
echo "generating scancodes for rust..."
TABLE_PATH="./data/scan_code_table.txt"
INTERMEDIATE_TABLE_PATH="./data/xset2_scan_codes.txt"

cut -f 6,9 $TABLE_PATH >"$INTERMEDIATE_TABLE_PATH"

declare -A LOOKUP_TBLE

{
    set -o noglob
    read -r LINE #IGNORE FIRST LINE
    while read -r LINE; do
        # SCAN_CODE=$(echo "${LINE}" | cut -f1)
        # KEY_LIST=$(echo "${LINE}" | cut -f2)
        # ARR=($KEY_LIST);
        # LEN=${#ARR[@]};
        # # printf "<SCODE=%s\tKEYS=%s\tLEN=%d>\n" "$SCAN_CODE" "$KEY_LIST" "$LEN"
        # for ((k=0;k<$LEN;k++)); do
        #     KEY=${ARR["$k"]};
        #     # echo "$KEY"
        #     LOOKUP_TBLE["$KEY"]="$SCAN_CODE";
        # done;
        SCAN_CODE=$(echo "${LINE}" | cut -f1)
        KEY_LIST=$(echo "${LINE}" | cut -f2)
        printf "%s=%s,\n"  "$KEY_LIST" "$SCAN_CODE"
    done
    set +o noglob
} <"$INTERMEDIATE_TABLE_PATH"
