#!/bin/bash
set flag=0
while read line;
do 
    echo $line
    if [flag==1];then
    sleep 5
    fi
    cargo run --bin $line &
done < setting.txt
echo "all servers opened"
