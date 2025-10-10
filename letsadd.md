 Image gallery

Use image galleries to embed images to the page. Images can be either hyphae or external files. In the example below you can replace the URL with a hypha's name. If that hypha is an image, the image will be shown.

You can write a description for the image and specify its size.

    img {
    https://bouncepaw.com/mushroom.jpg
    https://bouncepaw.com/mushroom.jpg {
    	Description //here//
    }
    https://bouncepaw.com/mushroom.jpg | 100 { Size }
    https://bouncepaw.com/mushroom.jpg | 50*50
    }

    Description here

    Size

    Square

Gallery layout

Set gallery layout to specify how your gallery is placed.

There are three layouts: normal (the default), grid and side.

Specify the layout after img and before {. If you do not write any of them, normal will be used.

img grid {
   https://bouncepaw.com/mushroom.jpg
   https://bouncepaw.com/mushroom.jpg
}

img side {
   https://bouncepaw.com/mushroom.jpg | 200
   https://bouncepaw.com/mushroom.jpg | 200
}

This text is wrapped.

This text is wrapped.
List

Lists are used for sequential or tree data. They are quite popular.

Mark each list entry with an asterisk and a space:

    * one
    * two
    * three

        one

        two

        three

If you place dots after the asterisks, the list becomes numbered:

    *. one
    *. two
    *. three

        one

        two

        three
