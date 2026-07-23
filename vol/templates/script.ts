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

let currentImageUrl: string | undefined

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
    if (currentImageUrl !== undefined) {
        URL.revokeObjectURL(currentImageUrl)
    }

    currentImageUrl = image
    mainImage.src = currentImageUrl
    seedDisplay.textContent = seed
    showImage()
}

generateBtn.addEventListener("click", renderImage)

renderImage().catch(console.error)