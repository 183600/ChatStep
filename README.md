# ChatStep

#### 介绍
- 现在未必已经实现基本功能
- 支持在与AI对话时执行shell命令
- 用free qwq的api
- 一个在linux下运行(其他一些系统可能也可以)的cli的rust程序
- [QQ群](http://qm.qq.com/cgi-bin/qm/qr?_wv=1027&k=XE6sZHD2Bi1IFKZyZ_hMa4T9UsFTMeBD&authKey=sCB6CvS4j%2BxPSlfh3BNR2%2F%2FTr%2BJ45T6Uku2YvmiyScPISHjiWSMZU%2BQMJJzh1EdW&noverify=0&group_code=1035618715)
- [Discord](https://discord.gg/MepDu9XJnd)
- [matrix](https://matrix.to/#/#chat-step:matrix.org)
#### 原理
接收到用户发送的信息之后，如果是一个指令并且完成这个指令如果需要给ai发送执行其他shell命令的结果（否则就直接把用户发送的信息发送给ai，然后把结果显示给用户），就让ai拆解成多个步骤，例如用户输入了“获取当前目录的内容并分析”，就先让ai把它拆成“linux下获取当前目录的结构”和“分析获取到的结果<结果>”这两个待会要给ai发送的信息，到这一步会显示接下来的步骤让用户确认，再问ai“获取当前目录的结构的shell命令是什么”，然后执行ai回答的shell命令，假设是ls -l，获取到total 16
-rw-rw---- 1 root everybody 262 2023-09-29 12:23 manifest.json
-rw-rw---- 1 root everybody 944 2023-09-16 20:57 twotone_info_black_48dp.png，最后把“分析获取到的结果<结果>”里的<结果>替换成total 16
-rw-rw---- 1 root everybody 262 2023-09-29 12:23 manifest.json
-rw-rw---- 1 root everybody 944 2023-09-16 20:57 twotone_info_black_48dp.png，把替换结果发送给ai，然后再把ai输出的结果显示给用户，上面是个例子，以此类推