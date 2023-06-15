## WordCloud ☁
#### A simplified way to create Wordclouds in Rust.

### Simple Example

   ```rust
   use wordcloud::*;
   use wordcloud::font::*;

   let mut font_data = Vec::from(include_bytes!("font.ttf") as &[u8]);
   let font_set = FontSetBuilder::new()
       .push(Font::from_data(&mut font_data).unwrap())
       .build();

   let output_dimensions = Dimensions::from_wh(1000, 1000);
   let wc = WordCloudBuilder::new()
       .dimensions(output_dimensions)
       .font(&font_set)
       .build()
       .unwrap();

   let input_text = io::read_string_from_file("input_text.txt").unwrap();
   let ranked = RankedWords::rank(input_text.split_whitespace().collect());

   wc.write_content(ranked, 2000);
   wc.export_rendered_to_file("output.svg").unwrap();
   ```

## Using Stopwords

   ```rust
   use wordcloud::*;
   use wordcloud::font::*;

   // Create the default StopWords set using the "stopwords" crate feature
   let stop_words = StopWords::default();

   let input_text = io::read_string_from_file("input_text.txt").unwrap();
   let ranked = RankedWords::rank(
       input_text
           .split_whitespace()
           // filter out the StopWords using the iterator function
           // a similar named function also exists for rayon's ParallelIterators
           .filter_stop_words(&stop_words)
           // filter out everything that is just a number
           .filter(
               |x| x.parse::<f64>().is_err()
           )
           .collect()
   );

   // ...
   ```

## Using Background Images

   ```rust
   use wordcloud::*;

   // ...

   let image = image::load_from_memory(test_image).expect("image load failed");
   let wc = WordCloudBuilder::new()
       .dimensions(output_dimensions)
       .font(&font_set)
       .image(&image)
       .build()
       .unwrap();

   // ...
   ```
