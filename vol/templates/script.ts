// npx tsc
const seedInput = document.getElementById("seed-input") as HTMLInputElement
const seedDisplay = document.getElementById("seed-value") as HTMLDivElement
const generateBtn = document.getElementById("generate-btn") as HTMLButtonElement

const downloadBtn = document.getElementById("download-btn") as HTMLButtonElement
const printBtn = document.getElementById("print-btn") as HTMLButtonElement

const mainImage = document.getElementById("main-image") as HTMLImageElement
const imagePlaceholder = document.getElementById("image-placeholder") as HTMLDivElement


const generateUrl = new URL("/generate", window.location.origin)
const getSeedUrl = new URL("/getSeed", window.location.origin)
const getImageUrl = new URL("/getImage", window.location.origin)
const getPrintImageUrl = new URL("/getPrintImage", window.location.origin)

let currentImageUrl: string | undefined
let currentPrintImageUrl: string | undefined

async function getSeed(): Promise<string> {
    const response = await fetch(getSeedUrl)

    if (!response.ok) {
        throw new Error(`Failed to get seed: ${response.status}`)
    }

    return await response.text()
}

async function getImage(): Promise<string> {
    const response = await fetch(getImageUrl)

    if (!response.ok) {
        throw new Error(`Failed to get image: ${response.status}`)
    }

    const imageBlob = await response.blob()
    const imageUrl = URL.createObjectURL(imageBlob)
    if (currentImageUrl !== undefined) {
        URL.revokeObjectURL(currentImageUrl)
    }
    currentImageUrl = imageUrl

    return imageUrl
}

async function getPrintImage(): Promise<string> {
    const response = await fetch(getPrintImageUrl)

    if (!response.ok) {
        throw new Error(`Failed to get print image: ${response.status}`)
    }

    const imageBlob = await response.blob()
    const imageUrl = URL.createObjectURL(imageBlob)
    if (currentPrintImageUrl !== undefined) {
        URL.revokeObjectURL(currentPrintImageUrl)
    }
    currentPrintImageUrl = imageUrl

    return imageUrl
}

async function generateImage() {
    const response = await fetch(generateUrl, {
        method: "POST",
        headers: {
            "Content-Type": "text/plain",
        },
        body: seedInput.value
    })

    if (!response.ok) {
        throw new Error(`Failed to generate: ${response.status}`)
    }
}

async function printImage(): Promise<void> {
    const win = window.open("", "_blank")

    if (win === null) {
        return
    }

    const image = win.document.createElement("img")
    image.src = await getPrintImage()

    const container = win.document.createElement("div")
    container.appendChild(image)

    const style = win.document.createElement("style")
    style.textContent = `
        @page {
            size: A4;
            margin: 0;
        }

        html, body {
            margin: 0;
            width: 210mm;
            height: 297mm;
        }

        .container {
            width: 210mm;
            height: 297mm;
            display: flex;
            justify-content: center;
            align-items: center;
            overflow: hidden;
        }

        img {
            max-width: 100%;
            max-height: 100%;
            object-fit: contain;
        }
    `

    container.className = "container"

    win.document.head.appendChild(style)
    win.document.body.appendChild(container)

    image.onload = () => {
        win.print()
    }
}

function hideImage(): void {
    imagePlaceholder.style.display = "flex"
    mainImage.style.display = "none"
}
function showImage(): void {
    mainImage.style.display = "flex"
    imagePlaceholder.style.display = "none"
}

async function renderImage(): Promise<void> {
    hideImage()
    await generateImage()

    const seed = await getSeed()
    const image = await getImage()


    currentImageUrl = image
    mainImage.src = currentImageUrl
    seedDisplay.textContent = seed
    showImage()
}

async function downloadImage() {
    const image = await getImage()
    const seed = await getSeed()

    const a = document.createElement("a")
    a.href = image
    a.download = `tucan_${seed}.png`
    a.click()
}

generateBtn.addEventListener("click", renderImage)
printBtn.addEventListener("click", printImage)
downloadBtn.addEventListener("click", downloadImage)

seedInput.addEventListener("input", () => {
    seedInput.value = seedInput.value.replace(/[^A-Fa-f0-9]/g, "")
})

renderImage().catch(console.error)