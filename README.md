### Overview

```
                                          ------> worker thread
              send job                    ------> worker thread
 main-thread ----------> buffered channel ------> worker thread
                                          ------> worker thread
                                          ------> worker thread
```
