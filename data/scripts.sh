#!/bash

while read -r LINE; do CHAR=${LINE[0]}; NUM= printf '%s=%d,\n' $LINE "'$CHAR" ; done < ./data/output.txt