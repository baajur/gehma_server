#!/bin/bash

if [$1 = ""]
then
	echo "Please add a version like v0.x.0"
	exit -1
fi

echo "Building kper/gehma:$1" \
&& docker build -t kper/gehma:$1 .  \
&& echo "Saving kper/gehma:$1 to ~/images/gehma_$1" \
&& docker save kper/gehma:$1 -o ~/images/gehma_$1  \
&& echo "Tar ~/images/gehma_$1" \
&& tar -cvf ~/images/gehma_$1.tar.gz ~/images/gehma_$1 \
&& chown kper:kper ~/images/gehma_$1.tar.gz\
&& rm ~/images/gehma_$1 \
&& echo "Build finished"
