sudo docker build -t kper/gehma:$1 . && cd .. && sudo docker save kper/gehma:$1 -o gehma_$1 && sudo chown kper:kper gehma_$1 && bzip2 gehma_$1
