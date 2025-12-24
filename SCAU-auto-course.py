from selenium import webdriver
import json
import time
import os, sys
from selenium.webdriver.edge.service import Service

base_dir = os.path.dirname(sys.executable)
driver_path = os.path.join(base_dir, "msedgedriver.exe")
service = Service(driver_path)

print("使用教程:")
print("-------------------------------------------------")
print("前期准备:")
print("1. 安装 Edge/Chromium 浏览器/确保drive在程序运行目录下")
print("2. 确保选课意向中有课程，脚本会自动持续提交意向选课里的课程")
print("-------------------------------------------------")
print("操作教程:")
print("1. 在弹出的浏览器界面，手动登录教务系统")
print("2. 登录完成后，返回程序回车继续，输入想要持续提交选课的时间（秒），按回车开始")
print("-------------------------------------------------")
print("输入回车开始执行")
input()

options = webdriver.EdgeOptions()
options.add_argument("--log-level=3")
options.add_argument("--disable-logging")
options.add_argument("--disable-usb-discovery")
options.add_argument("--disable-blink-features=AutomationControlled")
options.add_experimental_option("excludeSwitches", ["enable-logging"])

driver = webdriver.Edge(service=service, options=options)

driver.get("https://jwzf.scau.edu.cn")
input("请在弹出的浏览器界面，手动登录教务系统，登录完成后按回车继续...")

time.sleep(1)
current_url = driver.current_url
zt=True
if "login" in current_url:
    print("请在弹出的浏览器界面，手动登录教务系统，登录完成后按回车继续...")
    zt=False
while zt==False:
   current_url = driver.current_url
   if "login" in current_url:
    input("请在弹出的浏览器界面，手动登录教务系统，登录完成后按回车继续...")
   else:
      zt=True
      break

js2 = """
return fetch(
  "https://jwzf.scau.edu.cn/jwglxt/xsxk/zzxkyzb_cxWdgwcZzxkYzb.html?doType=query&gnmkdm=N253512",
  {
    method: "POST",
    headers: {
      "accept": "application/json, text/javascript, */*; q=0.01",
      "content-type": "application/x-www-form-urlencoded;charset=UTF-8",
      "x-requested-with": "XMLHttpRequest"
    },
    body: "xkxnm=2025&xkxqm=12&_search=false&queryModel.showCount=15&queryModel.currentPage=1&queryModel.sortName=zjsj+&queryModel.sortOrder=asc&time=0",
    credentials: "include"
  }
).then(res => res.json());
"""



print("获取课程json:")
resultjs=driver.execute_script(js2)
print(json.dumps(resultjs, ensure_ascii=False, indent=2))
ids2 = [item["xkgwcb_id"] for item in resultjs["items"]]
kcmc = [item["kcmc"] for item in resultjs["items"]]
kklxmc=[item["kklxmc"] for item in resultjs["items"]]
count=len(ids2)
if(count==0):
  print("未获取到课程id,无法执行下一步")
  input("按回车结束")
  driver.quit()
  sys.exit(0)
ids3 = ",".join(ids2)
print("选课意向中获取到以下课程")
for i in range(count):
  print(f"课程{i+1}: {kcmc[i]}  类型: {kklxmc[i]}  id: {ids2[i]}")


js_fetch = f"""
return fetch(
  "https://jwzf.scau.edu.cn/jwglxt/xsxk/zzxkyzbjk_xkBcZyZzxkYzbFromCart.html?gnmkdm=N253512",
  {{
    method: "POST",
    headers: {{
      "Accept": "application/json, text/javascript, */*; q=0.01",
      "Content-Type": "application/x-www-form-urlencoded;charset=UTF-8",
      "X-Requested-With": "XMLHttpRequest"
    }},
    body: "ids={ids3}",
    credentials: "include"
  }}
).then(r => r.json());
"""
stime=float(input("即将对这些课程抢课，输入持续抢课时间（秒）:"))
start = time.monotonic()
count=0
while time.monotonic() - start < stime:
    
  result = driver.execute_script(js_fetch)
  print("已发包次数:",count)
  count+=1
  print(json.dumps(result, ensure_ascii=False, indent=2))

print("结束")

input("👉 按回车关闭浏览器")
driver.quit()
