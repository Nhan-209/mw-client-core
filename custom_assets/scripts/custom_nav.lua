-- ===================================================================
--  MW Custom Client - Custom Navigation & UI Enhancer (Lua 5.1)
-- ===================================================================
print("[MW-Client-Core] Loading custom navigation script...")

local function initCustomNav()
    local UIMgr = GetInst("MiniUIManager")
    local JumpUtils = G_MainLobbyJumpUtils

    print("[MW-Client-Core] Custom UI Navigation initialized.")
end

-- Chờ môi trường game và FairyGUI nạp xong
if threadpool and threadpool.work then
    threadpool:work(function()
        initCustomNav()
    end)
else
    initCustomNav()
end
