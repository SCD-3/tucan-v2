// npx tsc
const seedInput = document.getElementById("seed-input") as HTMLInputElement
const seedDisplay = document.getElementById("seed-display") as HTMLDivElement
const generateBtn = document.getElementById("generate-btn") as HTMLButtonElement

const downloadBtn = document.getElementById("download-btn") as HTMLButtonElement
const printBtn = document.getElementById("print-btn") as HTMLButtonElement

const mainImage = document.getElementById("main-image") as HTMLImageElement
const imagePlaceholder = document.getElementById("image-placeholder") as HTMLDivElement


const setSeedUrl = new URL("/setSeed", window.location.origin)
const getSeedUrl = new URL("/getSeed", window.location.origin)
const getImageUrl = new URL("/getImage", window.location.origin)


async function setSeed(seed: String): Promise<void> {
    const value = seedInput.value

    const response = await fetch(setSeedUrl, {
        method: "POST",
        headers: {
            "Content-Type": "text/plain",
        },
        body: value
    })

    if (!response.ok) {
        throw new Error(`Failed to set seed: ${response.status}`)
    }
}

async function getSeed(): Promise<string> {
    const response = await fetch(getSeedUrl)

    if (!response.ok) {
        throw new Error(`Failed to get seed: ${response.status}`)
    }

    return await response.text()
}

async function getImage(): Promise<URL> {
    const response = await fetch(getImageUrl)

    if (!response.ok) {
        throw new Error(`Failed to get image: ${response.status}`)
    }

    const imageBlob = await response.blob()
    const imageUrl = new URL(URL.createObjectURL(imageBlob))
    return imageUrl
}

function hideImage(): void {
    imagePlaceholder.style.display = "flex"
    mainImage.style.display = "none"
}
function showImage(): void {
    mainImage.style.display = "flex"
    imagePlaceholder.style.display = "none"
}