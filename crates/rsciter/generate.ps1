$code = (bindgen --default-enum-style rust `
        --newtype-enum "SCRIPT_RUNTIME_FEATURES|SOM_EVENTS|OUTPUT_.*|VALUE_.*|.*_FLAGS|.*_flags" `
        --bitfield-enum "EVENT_GROUPS" `
        --allowlist-file=".*sciter.*\.h" `
        --allowlist-file=".*value\.h" `
        --blocklist-function "Sciter.*" `
        --opaque-type "IUnknown" `
        --blocklist-type "tag.*|WPARAM|LPARAM|LRESULT|MSG|HWND|HWND__|RECT|POINT|SIZE|.*_PTR" `
	--blocklist-type "LPRECT|LPPOINT|LPSIZE" `
        --blocklist-item "TRUE|FALSE|SCITER_DLL_NAME" `
        --no-layout-tests --no-doc-comments --raw-line "use super::*;" `
        .\sciter-x-api.h 
) | Out-String
$code = $code -replace "pub SciterCreateNSView: LPVOID", "pub SciterCreateNSView: ::std::option::Option<unsafe extern `"C`" fn(frame: LPRECT) -> HWND>"
$code = $code -replace "pub SciterCreateWidget: LPVOID", "pub SciterCreateWidget: ::std::option::Option<unsafe extern `"C`" fn(frame: LPRECT) -> HWND>"

Set-Content -Path "generated.rs" -Value $code