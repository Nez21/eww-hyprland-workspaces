# Eww Hyprland Workspaces

A small tool to sync Hyprland workspace status to Eww

## Usage

Use with eww literal widget, example:

```yuck
(deflisten workspace "scripts/workspace")
(defwidget workspaces []
  (literal :content workspace))
```

## Config

Name as `config.yaml` and place at same directory with executable

```yaml
workspaces:
   1: '一'
   2: '二'
   3: '三'
   4: '四'
   5: '五'
   6: '六'
   7: '七'
   8: '八'
   9: '九'
   10: '〇'
template: |-
   (eventbox :onscroll "echo {} | sed -e 's/up/-1/g' -e 's/down/+1/g' | xargs hyprctl dispatch workspace"
      (box :class "workspaces" :orientation "h" :spacing 5
        {body}
      )
   )
bodyTemplate: |-
   (button :onclick "hyprctl dispatch workspace {id}" :class "workspace {state}" "{icon}")
```
