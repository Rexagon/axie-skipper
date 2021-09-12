const injectScript = () => {
    try {
        const container = document.head || document.documentElement
        const scriptTag = document.createElement('script')
        scriptTag.src = chrome.runtime.getURL('inpage.js')
        scriptTag.setAttribute('async', 'false')
        container.insertBefore(scriptTag, container.children[0])
        container.removeChild(scriptTag)
    } catch (e) {
        console.error('Provider injection failed', e)
    }
}

injectScript()
