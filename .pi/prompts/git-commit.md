---
description: Commit current work with conventional commit message
---

Сделай git-коммит текущей работы:

1. Проверь `git status` — убедись что нет мусора (target/, node_modules/)
2. Если .gitignore не настроен — создай:
   ```
   /target
   .pi/
   ```
3. `git add -A`
4. Определи тип изменения:
   - Новый код → `feat(<crate>): <описание>`
   - Исправление → `fix(<crate>): <описание>`
   - Архивация → `chore(openspec): archive <change-name>`
   - Тесты → `test(<crate>): <описание>`
5. В теле коммита перечисли ключевые файлы и что сделано
6. Выполни `git commit`
7. Покажи результат: `git log --oneline -1`
