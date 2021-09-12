async function requestMessage() {
    const response = await fetch('https://axieinfinity.com/graphql-server-v2/graphql', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            "operationName": "CreateRandomMessage",
            "variables": {},
            "query": "mutation CreateRandomMessage {\n  createRandomMessage\n}\n"
        }),
    }).then(r => r.json());

    return response.data.createRandomMessage;
}

async function signMessage(message) {
    return await fetch('http://127.0.0.1:3030/sign', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            accountId: 0,
            message
        }),
    }).then(r => r.json());
}

async function createAccount(params) {
    const request = {
        "operationName": "CreateAccessTokenWithSignature",
        "variables": {
            "input": {
                "mainnet": "ronin",
                "owner": params.owner,
                "message": params.message,
                "signature": params.signature
            }
        },
        "query": "mutation CreateAccessTokenWithSignature($input: SignatureInput!) {\n  createAccessTokenWithSignature(input: $input) {\n    newAccount\n    result\n    accessToken\n    __typename\n  }\n}\n"
    }

    const response = await fetch('https://axieinfinity.com/graphql-server-v2/graphql', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
    }).then(r => r.json());

    return response.data.createAccessTokenWithSignature.accessToken;
}

async function onSkipClicked() {
    const message = await requestMessage();
    const params = await signMessage(message);
    const accessToken = await createAccount(params);

    localStorage.setItem('accessToken', accessToken);
    window.location = 'https://marketplace.axieinfinity.com/profile/dashboard'
}

function onFormLoaded(form) {
    const button = document.createElement('button');
    button.className += 'px-20 py-8 relative rounded transition focus:outline-none border w-full text-white border-gray-2 hover:border-gray-1 active:border-gray-3 bg-gray-5 hover:bg-gray-4 active:bg-gray-6';
    button.innerText = 'SKIP';
    button.addEventListener('click', onSkipClicked);

    const wrapper = document.createElement('div');
    wrapper.className = 'mb-20';
    wrapper.appendChild(button);

    form.appendChild(wrapper);
}

window.addEventListener('load', function () {
    const observer = new MutationObserver(function (mutations, me) {
        const form = document.querySelector('#__next > div > .mx-auto');
        if (form) {
            onFormLoaded(form);
            me.disconnect();
        }
    });

    observer.observe(document, {
        childList: true,
        subtree: true
    });
})
