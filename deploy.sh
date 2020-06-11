#!/bin/bash

if [$1 = ""]
then
	echo "Please add a version like v0.x.0"
	exit -1
fi

./build.sh $1 \
&& echo "Begin transfer to gehma_prod:/IMAGES/gehma_$1" \
&& sudo scp ~/images/gehma_$1.tar.gz kper@gehma_prod:/IMAGES \
&& echo "Transfer finished"
