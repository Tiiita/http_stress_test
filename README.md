# Stress Tester

### Important>
You may encounter issues with the ```-b``` or ```--body``` flag, if passing in json. Json needs to passed in as following:
```-b '{\"key\": \"value\"}'``` otherwise most shells won't parse the json correctly!

This is a http stress tester which can send many http requests at once.
Unfortunately it gets flagged by Microsoft Antivirus and the .exe will be removed automatically and instantly after running. 
I do not have a fix for that. So would need to clone the repo and run ```cargo run -r -- --help```

**PLEASE DO NOT ABUSE THIS BECAUSE IT CAN BLOW UP THE TRAFFIC AND CAN SLOW DOWN POOR SERVERS!**
I TAKE **NO** RESPONSIBILITY!
